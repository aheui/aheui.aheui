// prototype for aheui-aheui self-interpreter in Rust
// 2015-04-01 Kang Seonghoon

#![feature(io, exit_status)]

#[macro_use] extern crate log;
extern crate env_logger;

use std::char;
use std::io;
use std::env;
use std::io::{Read, Write};
use std::collections::VecDeque;

fn main() {
    env_logger::init().unwrap();

    let mut stdin = io::stdin().chars();
    let mut ungetced = None;

    macro_rules! getch {
        () => ({
            ungetced.take().unwrap_or_else(|| {
                stdin.next().and_then(|c| c.ok()).map_or(-1, |c| c as i32)
            })
        })
    }

    macro_rules! getint {
        () => ({
            let mut c = getch!();
            while c == b' ' as i32 || c == b'\t' as i32 || c == b'\n' as i32 {
                c = getch!();
            }
            let mut s = String::new();
            if c == b'+' as i32 || c == b'-' as i32 {
                s.push(c as u8 as char);
                c = getch!();
            }
            while b'0' as i32 <= c && c <= b'9' as i32 {
                s.push(c as u8 as char);
                c = getch!();
            }
            while c == b' ' as i32 || c == b'\t' as i32 || c == b'\n' as i32 {
                c = getch!();
            }
            ungetced = Some(c);
            debug!("{}", s);
            s.parse::<i32>().ok().expect("got something other than number")
        })
    }

    // main program space queue
    // (2 (value)* 0)* 0
    // value is offset by 21 * 28 for syllables and set to 28 for non-syllables
    let mut q: VecDeque<i32> = VecDeque::new();

    loop {
        let v = getch!();
        if v == 10 {
            q.push_back(0);
            q.push_back(2);
            continue;
        }
        if v <= 0 { // allows NUL as a code-input separator
            break;
        }
        // 0xac00 = push8 push8 push8 push9 push9 mul push5 add mul mul mul (11)
        let v = v - 0xac00;
        if v < 0 {
            q.push_back(28); // 개 (the first no-op character)
            continue;
        }
        // 11172 = push3 push4 push7 push7 push2 push8 push9 add add mul mul mul mul (13)
        // 11172 = push7 push8 mul push5 add dup mul push3 mul push9 add (11)
        // 11172 = push8 push5 mul dup mul push4 sub push7 mul (9)
        if v < 19 * 21 * 28 {
            let z = v % 28;
            let zdelta;
            loop {
                if z < 9 {
                    if z < 1 { zdelta = 4; break; } // can be optimized to z == 0
                    if z < 2 { zdelta = 0; break; }
                    if z < 4 { zdelta = 14; break; }
                    if z < 5 { zdelta = -2; break; }
                    if z < 7 { zdelta = 0; break; }
                    if z < 8 { zdelta = 17; break; }
                    zdelta = -1; break;
                }
                let z = z - 9;
                if z < 9 {
                    if z < 1 { zdelta = 0; break; } // can be optimized to z == 0
                    if z < 3 { zdelta = 1; break; }
                    if z < 4 { zdelta = -2; break; }
                    if z < 7 { zdelta = 0; break; }
                    zdelta = 2; break;
                }
                let z = z - 9;
                if z < 1 { zdelta = -10; break; } // can be optimized to z == 0
                if z < 2 { zdelta = -16; break; }
                if z < 3 { zdelta = 0; break; }
                if z < 4 { zdelta = -21; break; }
                if z < 5 { zdelta = 3; break; }
                if z < 6 { zdelta = -2; break; }
                if z < 7 { zdelta = 2; break; }
                if z < 9 { zdelta = -3; break; }
                zdelta = 0; break;
            }
            q.push_back(v + 21 * 28 + zdelta);
        } else {
            q.push_back(28);
        }
    }
    q.push_back(0);
    q.push_back(0);
    q.push_back(2); // the beginning of the first row

    info!("queue: {:?}", q);

    // zipper-like tape for stacks
    // there are exactly 26 stacks and one queue, thus 27 zeroes
    // the current stack is always fetched from h
    // _ 0 # _ 1 # 2 # 3 # | # 4 # 5 # 6 _ _ _ ... _ _
    // <---> <-----------> | <-----------> ^ ^     ^ ^
    //  S0        S1       |      S2      S3 S4  S25 Q
    //         (selected)  |   (reversed)
    //
    // _ stands for the number 2 and signals the end of stack/queue
    // # stands for the number 0 and signals the next element
    //
    // queue is special cased
    // _ _ ... _ _ 3 # 2 # 1 # | # 4 # 5 # 6 _
    // ^ ^+    ^ <--------------------------->
    // S0 S1 S25               Q
    //
    // insertion occurs at h, deletion occurs at t
    // initial               _ | _
    // push 1            _ 1 # | _
    // push 2        _ 1 # 2 # | _
    // push 3    _ 1 # 2 # 3 # | _
    // rearrange for pop     _ | # 1 # 2 # 3 _
    // pop 1                 _ | # 2 # 3 _
    // push 4            _ 4 # | # 2 # 3 _
    // push 5        _ 4 # 5 # | # 2 # 3 _
    // rearrange for switch  _ | # 2 # 3 # 4 # 5 _
    // * rearrange only happens when t is empty or it switches back from the queue
    let mut h: Vec<i32> = Vec::new(); // head stack
    let mut t: Vec<i32> = Vec::new(); // tail stack
    let mut s = 2 * 4; // queue=0, stack=2,4,6,8...2*27 (rearranged)

    //const SENTINEL: i32 = 2;
    const SENTINEL: i32 = 22222;

    const DUMP_LIMIT: usize = 25;
    macro_rules! dump_stack {
        ($e:expr) => (
            info!("{} ({}): {}{:?} | {:?}{}", $e, h.len() + t.len(),
                  if h.len() < DUMP_LIMIT { "" } else { "..." },
                  if h.len() < DUMP_LIMIT { &h[..] } else { &h[h.len() - DUMP_LIMIT..] },
                  (if t.len() < DUMP_LIMIT { &t[..] } else { &t[t.len() - DUMP_LIMIT..] })
                    .iter().rev().cloned().collect::<Vec<_>>(),
                  if t.len() < DUMP_LIMIT { "" } else { "..." })
        )
    }

    for _ in 0..s/2+1 { h.push(SENTINEL); }
    for _ in s/2+1..28 { t.push(SENTINEL); }

    // for the convenience, we scale all coords by two (there is no odd one)
    let mut r = 0;
    let mut c = 0;
    let mut dr = 4 + 2;
    let mut dc = 4 + 0;

    // possible with dup then move
    let mut v = *q.front().unwrap();

    loop {
        if false { // same trace format as asm
            print!("{} {}  {} {}  {}  {}  ", r/2,c/2,dr/2-2,dc/2-2,s/2,v);
            let zz = h.len(); if h[zz-1] == 0 { println!("{}", h[zz-2]); } else { println!("-"); }
        }

        info!("r: {} ({}), c: {} ({}), s: {} ({}), command: {} (op: {}, dir: {}, arg: {})",
              r/2, (dr-4)/2, c/2, (dc-4)/2, s/2,
              ["ㅇ","ㄱ","ㄴ","ㅅ","","ㄵ","ㄶ","ㄹ","ㅄ","ㄺ","ㄽ","ㄻ","ㄼ","ㄾ","ㄿ","ㅀ",
               "ㄲ","ㄳ","ㅁ","ㅂ","ㅆ","ㅊ","ㅌ","ㅍ","ㄷ","ㅈ","ㅋ","ㅎ"][(s/2) as usize],
              v,
              ["","ㄱ","ㄲ","ㄴ","ㄷ","ㄸ","ㄹ","ㅁ","ㅂ","ㅃ","ㅅ","ㅆ","ㅇ",
               "ㅈ","ㅉ","ㅊ","ㅋ","ㅌ","ㅍ","ㅎ"][(v/21/28) as usize],
              ["ㅏ","ㅐ","ㅑ","ㅒ","ㅓ","ㅔ","ㅕ","ㅖ","ㅗ","ㅘ","ㅙ","ㅚ",
               "ㅛ","ㅜ","ㅝ","ㅞ","ㅟ","ㅠ","ㅡ","ㅢ","ㅣ"][(v/28%21) as usize],
//            ["","ㄱ","ㄲ","ㄳ","ㄴ","ㄵ","ㄶ","ㄷ","ㄹ","ㄺ","ㄻ","ㄼ","ㄽ","ㄾ","ㄿ","ㅀ",
//             "ㅁ","ㅂ","ㅄ","ㅅ","ㅆ","ㅇ","ㅈ","ㅊ","ㅋ","ㅌ","ㅍ","ㅎ"][(v%28) as usize]);
              ["ㅇ","ㄱ","ㄴ","ㅅ","","ㄵ","ㄶ","ㄹ","ㅄ","ㄺ","ㄽ","ㄻ","ㄼ","ㄾ","ㄿ","ㅀ",
               "ㄲ","ㄳ","ㅁ","ㅂ","ㅆ","ㅊ","ㅌ","ㅍ","ㄷ","ㅈ","ㅋ","ㅎ"][(v%28) as usize]);
        dump_stack!("stack");

        // TODO stack verification

        // handle the case that this cell does exist
        if v != 0 {
            let z = v % 28;
            let xy = v / 28;
            let y = xy % 21;
            let x = xy / 21;

            loop {
                if y < 8 {
                    if y == 0 { dr = 4 + 0; dc = 4 + 2; break; }
                    if y == 2 { dr = 4 + 0; dc = 4 + 4; break; }
                    if y == 4 { dr = 4 + 0; dc = 4 - 2; break; }
                    if y == 6 { dr = 4 + 0; dc = 4 - 4; break; }
                    break;
                }
                let y = y - 8;
                if y < 9 {
                    if y == 0 { dr = 4 - 2; dc = 4 + 0; break; }
                    if y == 4 { dr = 4 - 4; dc = 4 + 0; break; }
                    if y == 5 { dr = 4 + 2; dc = 4 + 0; break; }
                }
                let y = y - 7;
                if y == 2 { dr = 4 + 4; dc = 4 + 0; break; }
                if y == 3 { dr = 8 - dr; break; }
                if y == 4 { dr = 8 - dr; dc = 8 - dc; break; }
                if y == 5 { dc = 8 - dc; break; }
                break;
            }

            let k; // nonzero when delta should reflect
            loop {
                if x == 0 || x == 12 { // empty or ㅇ "official" nop ()
                    // this is not strictly necessary but added as optimization
                    k = 0; break;
                }
                if x == 8 { // ㅂ push ()
                    if z < 9 {
                        if z == 0 { h.push(getint!()); }
                        else if z < 4 { h.push(2); }
                        else if z < 5 { h.push(0); }
                        else if z < 8 { h.push(5); }
                        else { h.push(6); }
                    } else {
                        let z = z - 9;
                        if z < 7 {
                            if z < 2 { h.push(7); }
                            else if z < 6 { h.push(9); }
                            else { h.push(8); }
                        } else {
                            let z = z - 9;
                            if z < 6 { h.push(4); }
                            else if z < 9 { h.push(3); }
                            else { h.push(getch!()); }
                        }
                    }
                    h.push(0);
                    k = 0; break;
                }
                if x == 10 { // ㅅ switch ()
                    let n = -1; // dummy
                    let mut delta = z * 2 - s;
                    let mut flag = z * 2; // -1 (move #1), 3 (move #2), positive even (switch)

                    // ** shared with ㅆ (including the rearrangmeent)
                    if s == 0 {
                        // rearrange the queue before moving around:
                        //
                        //           top -->   <-- top           --> top
                        //        head stack | tail stack      | scratch stack
                        //         _ 4 # 5 # | # 2 # 3 _ <...> | _
                        //             _ 4 # | # 2 # 3 _ <...> | _ 5 #
                        //                 _ | # 2 # 3 _ <...> | _ 5 # 4 #
                        //             _ 2 # | # 3 _ <...>     | _ 5 # 4 #
                        //         _ 2 # 3 # | _ <...>         | _ 5 # 4 #
                        //     _ 2 # 3 # 4 # | _ <...>         | _ 5 #
                        // _ 2 # 3 # 4 # 5 # | _ <...>         | _
                        // _ 2 # 3 # 4 # 5 # | <...>           |
                        //
                        // the last step (removing an excess _) is required to
                        // make the queue layout-compatible with other storages.

                        dump_stack!("stack before moving rearrange");
                        let mut a: Vec<i32> = Vec::new();
                        a.push(SENTINEL);
                        loop {
                            let fin = h.pop().unwrap();
                            if fin == 0 {
                                a.push(h.pop().unwrap());
                                a.push(0);
                            } else {
                                break;
                            }
                        }
                        h.push(SENTINEL);
                        loop {
                            let fin = t.pop().unwrap();
                            if fin == 0 {
                                h.push(t.pop().unwrap());
                                h.push(0);
                            } else {
                                break;
                            }
                        }
                        loop {
                            let fin = a.pop().unwrap();
                            if fin == 0 {
                                h.push(a.pop().unwrap());
                                h.push(0);
                            } else {
                                break;
                            }
                        }
                        dump_stack!("stack after moving rearrange");
                    }
                    loop {
                        while delta > 0 {
                            h.push(SENTINEL);
                            loop {
                                let fin = t.pop().unwrap();
                                if fin == 0 {
                                    h.push(t.pop().unwrap());
                                    h.push(0);
                                } else {
                                    break;
                                }
                            }
                            delta -= 2;
                        }
                        while delta < 0 {
                            t.push(SENTINEL);
                            loop {
                                let fin = h.pop().unwrap();
                                if fin == 0 {
                                    t.push(h.pop().unwrap());
                                    t.push(0);
                                } else {
                                    break;
                                }
                            }
                            delta += 2;
                        }
                        assert_eq!(delta, 0);

                        if flag < 0 {
                            h.push(n);
                            h.push(0);
                            delta = s - z * 2;
                            flag += 4; // -1 -> 3
                        } else {
                            if flag != 3 {
                                s = flag;
                            }
                            if s == 0 { t.push(SENTINEL); } // needs a sentinel
                            break;
                        }
                    }

                    k = 0; break;
                }

                let mut nfin;
                let mut n = -0xf00f;
                if s != 0 { // stack
                    nfin = h.pop().unwrap();
                    if nfin == 0 {
                        n = h.pop().unwrap();
                    } else {
                        h.push(SENTINEL);
                    }
                } else { // queue
                    nfin = t.pop().unwrap();
                    if nfin == 0 {
                        n = t.pop().unwrap();
                    } else {
                        // rearrange for pop:
                        //
                        //    top -->   <-- top
                        // head stack | tail stack
                        //  _ 4 # 5 # | _ <...>
                        //      _ 4 # | # 5 _ <...>
                        //          _ | # 4 # 5 _ <...>
                        t.push(SENTINEL);
                        dump_stack!("stack before popping rearrange");
                        loop {
                            let fin = h.pop().unwrap();
                            if fin == 0 {
                                t.push(h.pop().unwrap());
                                t.push(0);
                            } else {
                                h.push(SENTINEL);
                                break;
                            }
                        }
                        dump_stack!("stack after popping rearrange");

                        // retry
                        nfin = t.pop().unwrap();
                        if nfin == 0 {
                            n = t.pop().unwrap();
                        } else {
                            t.push(SENTINEL);
                        }
                    }
                }

                let x = x - 11;
                if x == 8 { // ㅎ exit (x*)
                    // switch to the empty stack and simulate the exit
                    let ex;
                    if nfin == 0 {
                        ex = Some(n);
                        env::set_exit_status(n);
                    } else {
                        ex = None;
                    }
                    info!("exit code: {:?}", ex);
                    return;
                }
                if x == 0 { // ㅆ move (x)
                    if nfin != 0 { k = 2; break; }

                    let mut delta = z * 2 - s;
                    let mut flag = -1;

                    // 1. switch to the storage (same to ㅅ)
                    // 2. push n
                    // 3. switch back to the original storage
                    // ㅅ, ㅆ-1 and ㅆ-3 are same and should be shared

                    // ** shared with ㅅ (including the rearrangmeent)
                    if s == 0 {
                        // rearrange the queue before moving around:
                        //
                        //           top -->   <-- top           --> top
                        //        head stack | tail stack      | scratch stack
                        //         _ 4 # 5 # | # 2 # 3 _ <...> | _
                        //             _ 4 # | # 2 # 3 _ <...> | _ 5 #
                        //                 _ | # 2 # 3 _ <...> | _ 5 # 4 #
                        //             _ 2 # | # 3 _ <...>     | _ 5 # 4 #
                        //         _ 2 # 3 # | _ <...>         | _ 5 # 4 #
                        //     _ 2 # 3 # 4 # | _ <...>         | _ 5 #
                        // _ 2 # 3 # 4 # 5 # | _ <...>         | _
                        // _ 2 # 3 # 4 # 5 # | <...>           |
                        //
                        // the last step (removing an excess _) is required to
                        // make the queue layout-compatible with other storages.

                        dump_stack!("stack before moving rearrange");
                        let mut a: Vec<i32> = Vec::new();
                        a.push(SENTINEL);
                        loop {
                            let fin = h.pop().unwrap();
                            if fin == 0 {
                                a.push(h.pop().unwrap());
                                a.push(0);
                            } else {
                                break;
                            }
                        }
                        h.push(SENTINEL);
                        loop {
                            let fin = t.pop().unwrap();
                            if fin == 0 {
                                h.push(t.pop().unwrap());
                                h.push(0);
                            } else {
                                break;
                            }
                        }
                        loop {
                            let fin = a.pop().unwrap();
                            if fin == 0 {
                                h.push(a.pop().unwrap());
                                h.push(0);
                            } else {
                                break;
                            }
                        }
                        dump_stack!("stack after moving rearrange");
                    }
                    loop {
                        while delta > 0 {
                            h.push(SENTINEL);
                            loop {
                                let fin = t.pop().unwrap();
                                if fin == 0 {
                                    h.push(t.pop().unwrap());
                                    h.push(0);
                                } else {
                                    break;
                                }
                            }
                            delta -= 2;
                        }
                        while delta < 0 {
                            t.push(SENTINEL);
                            loop {
                                let fin = h.pop().unwrap();
                                if fin == 0 {
                                    t.push(h.pop().unwrap());
                                    t.push(0);
                                } else {
                                    break;
                                }
                            }
                            delta += 2;
                        }
                        assert_eq!(delta, 0);

                        if flag < 0 {
                            h.push(n);
                            h.push(0);
                            delta = s - z * 2;
                            flag += 4; // -1 -> 3
                        } else {
                            if flag != 3 {
                                s = flag;
                            }
                            if s == 0 { t.push(SENTINEL); } // needs a sentinel
                            break;
                        }
                    }

                    k = 0; break;
                }
                if x == -4 { // ㅁ pop (x)
                    if nfin != 0 { k = 2; break; }
                    if z == 0 {
                        print!("{}", n);
                        io::stdout().flush().unwrap();
                    } else if z == 27 {
                        print!("{}", char::from_u32(n as u32).unwrap());
                        io::stdout().flush().unwrap();
                    }
                    k = 0; break;
                }
                if x == 4 { // ㅊ branch (x)
                    if nfin != 0 { k = 2; break; }
                    if n == 0 {
                        dr = 8 - dr;
                        dc = 8 - dc;
                    }
                    k = 0; break;
                }
                if x == -2 { // ㅃ dup (x)
                    if nfin != 0 { k = 2; break; }
                    // this differs from other commands that it has a knowledge about the storage
                    if s != 0 {
                        h.push(n);
                        h.push(0);
                        h.push(n);
                        h.push(0);
                    } else {
                        t.push(n);
                        t.push(0);
                        t.push(n);
                        t.push(0);
                    }
                    k = 0; break;
                }

                let mut mfin;
                let mut m = -0xf00f;
                if nfin == 0 {
                    if s != 0 { // stack
                        mfin = h.pop().unwrap();
                        if mfin == 0 {
                            m = h.pop().unwrap();
                        } else {
                            h.push(SENTINEL);
                            h.push(n);
                            h.push(0);
                        }
                    } else { // queue
                        // probably this code cannot be shared among two cases
                        // try to be as compact as possible
                        mfin = t.pop().unwrap();
                        if mfin == 0 {
                            m = t.pop().unwrap();
                        } else {
                            // rearrange for pop
                            t.push(SENTINEL);
                            dump_stack!("stack before popping rearrange");
                            loop {
                                let fin = h.pop().unwrap();
                                if fin == 0 {
                                    t.push(h.pop().unwrap());
                                    t.push(0);
                                } else {
                                    h.push(SENTINEL);
                                    break;
                                }
                            }
                            dump_stack!("stack after popping rearrange");

                            // retry
                            mfin = t.pop().unwrap();
                            if mfin == 0 {
                                m = t.pop().unwrap();
                            } else {
                                t.push(SENTINEL);
                                t.push(n);
                                t.push(0);
                            }
                        }
                    }
                } else {
                    mfin = nfin;
                }

                let x = x + 11;
                if x == 4 { // ㄷ add (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(m + n);
                    h.push(0);
                    k = 0; break;
                }
                if x == 13 { // ㅈ cmp (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(if m >= n { 1 } else { 0 });
                    h.push(0);
                    k = 0; break;
                }
                if x == 5 { // ㄸ mul (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(m * n);
                    h.push(0);
                    k = 0; break;
                }
                if x == 3 { // ㄴ div (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(m / n);
                    h.push(0);
                    k = 0; break;
                }
                if x == 6 { // ㄹ mod (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(m % n);
                    h.push(0);
                    k = 0; break;
                }
                if x == 17 { // ㅌ sub (x y)
                    if mfin != 0 { k = 2; break; }
                    h.push(m - n);
                    h.push(0);
                    k = 0; break;
                }
                if x == 18 { // ㅍ swap (x y)
                    if mfin != 0 { k = 2; break; }
                    // this differs from other commands that it has a knowledge about the storage
                    if s != 0 {
                        h.push(n);
                        h.push(0);
                        h.push(m);
                        h.push(0);
                    } else {
                        t.push(n);
                        t.push(0);
                        t.push(m);
                        t.push(0);
                    }
                    k = 0; break;
                }

                // no matching opcode, roll the stack back
                if mfin == 0 {
                    // essentially same except for the operating storage
                    if s != 0 { // stack
                        h.push(m);
                        h.push(0);
                        h.push(n);
                        h.push(0);
                    } else { // queue
                        t.push(m);
                        t.push(0);
                        t.push(n);
                        t.push(0);
                    }
                }
                k = 0; break;
            }

            if k != 0 {
                dr = 8 - dr;
                dc = 8 - dc;
            }
        }

        // skip to the next instruction
        let fullseek;

        if dr != 4 {
            // up (and temporarily down)
            assert!(dc == 4);

            // skip through the last line while calculating the last line number
            let mut lastr = r;
            loop {
                // move to queue should move the front value into the back
                // combined with front-only dups, this is fairly cheap
                let w = q.pop_front().unwrap();
                q.push_back(w);
                if w == 0 {
                    let hasnext = q.pop_front().unwrap();
                    q.push_back(hasnext);
                    if hasnext != 0 {
                        lastr += 2;
                    } else {
                        break;
                    }
                }
            }

            // wrapping around
            r += dr - 4;
            if r < 0 {
                r = lastr;
            } else if r > lastr {
                r = 0;
            }

            fullseek = 2;
        } else if dc != 4 {
            // left (and temporarily right)
            assert!(dr == 4);

            // skip until the next line
            let mut lastc = c - 2; // since the next item includes v itself
            assert_eq!(*q.front().unwrap(), v);
            loop {
                let w = q.pop_front().unwrap();
                q.push_back(w);
                if w != 0 {
                    lastc += 2;
                } else {
                    break;
                }
            }

            // we've got lastc, but still have to go through the entire file
            let hasnext = q.pop_front().unwrap(); // **
            q.push_back(hasnext);
            if hasnext != 0 {
                loop {
                    let w = q.pop_front().unwrap();
                    q.push_back(w);
                    if w == 0 {
                        let hasnext = q.pop_front().unwrap(); // **
                        q.push_back(hasnext);
                        if hasnext == 0 {
                            break;
                        }
                    }
                }
            }

            // wrapping around
            c += dc - 4;
            if c < 0 {
                c = lastc;
            } else if c > lastc {
                c = 0;
            }

            fullseek = 2;
        } else {
            panic!();
        }

        if fullseek != 0 {
            // skip r-1 rows
            let mut remainingr = r;
            loop {
                let hasnext = q.pop_front().unwrap();
                q.push_back(hasnext);
                // we push at least one row
                // and we will always clip `r` with the (positive) number of rows
                // so we cannot reach the end of file before we skip n rows
                assert!(hasnext != 0);
                if remainingr == 0 {
                    break;
                }
                remainingr -= 2;
                loop {
                    let w = q.pop_front().unwrap();
                    q.push_back(w);
                    if w == 0 { break; }
                }
            }

            // skip c-1 columns and set v
            let mut remainingc = c;
            loop {
                let w = *q.front().unwrap();
                if w == 0 {
                    // EOL reached first, do not advance further
                    v = w;
                    break;
                }
                if remainingc == 0 {
                    v = w;
                    break;
                }
                q.pop_front();
                q.push_back(w);
                remainingc -= 2;
            }
        } else {
            v = q.pop_front().unwrap();
            q.push_back(v);
        }
    }
}

