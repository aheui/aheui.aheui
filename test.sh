#!/bin/bash

require_rpaheui() {
    if [ -z "${RPAHEUI}" ]; then
        echo You need to set an '${RPAHEUI}' environment variable. >&2
        exit 1
    fi
}

case ${1} in
    proto) AHEUI=$(dirname "$0")/ahah;;
    asm) require_rpaheui; AHEUI="${RPAHEUI}/aheui-c --no-c $(dirname "$0")/aheui.aheuis";;
    aheui) require_rpaheui; AHEUI="${RPAHEUI}/aheui-c --no-c $(dirname "$0")/aheui.aheui";;
    *) echo "Usage: $0 {proto|asm|aheui} [testname ...]" >&1; exit 1
esac

git submodule init
git submodule update
if [ ${2} ]; then
    shift
    ds=snippets/${*}
else
    ds=snippets/*/
fi

success=0
fail=0
for d in $ds; do
    echo 'testset:' $d
    for f in $d/*.aheui; do
        fbase=`basename "$f" .aheui`
        echo -n "  test $fbase"...
        if [ -e "$d/$fbase".out ]; then
            if [ -e "$d/$fbase".in ]; then
                #out=`$AHEUI $f < $d/$fbase.in`
                out=`(cat $f <(echo -ne '\0') $d/$fbase.in) | $AHEUI`
            else
                #out=`$AHEUI $f`
                out=`$AHEUI <$f`
            fi
            exitcode=$?
            if [ -e "$d/$fbase".exitcode ]; then
                exitcodetest=1
                exitcodedata=`cat "$d/$fbase".exitcode`
            else
                exitcodetest=0
            fi
            outdata=`cat "$d/$fbase".out`
            if [ "$out" == "$outdata" ]; then
                if [ "$exitcodetest" == 1 ]; then
                    if [ "$exitcode" == "$exitcodedata" ]; then
                        success=$(($success + 1))
                        echo -e "\x1B[92msuccess!\x1B[0m"
                    else
                        fail=$(($fail + 1))
                        echo -e "\x1B[91mfail!\x1B[0m"
                        echo -e "    \x1B[92mexpected exitcode\x1B[0m $exitcodedata"
                        echo -e "    \x1B[91mactual exitcode\x1B[0m   $exitcode"
                    fi
                else
                    success=$(($success + 1))
                    echo -e "\x1B[92msuccess!\x1B[0m"
                fi
            else
                fail=$(($fail + 1))
                echo -e "\x1B[91mfail!\x1B[0m"
                echo -e "    \x1B[92mexpected\x1B[0m $outdata"
                echo -e "    \x1B[91mactual\x1B[0m   $(echo $out | cut -b -100)"
            fi
        else
            echo -e '\x1B[93moutput not found\x1B[0m'
        fi
    done
done

echo -e "test status: $success/\x1B[92m$(($success + $fail))\x1B[0m"
exit $fail
