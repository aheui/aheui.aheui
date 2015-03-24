#!/bin/bash

AHEUI=$(dirname "$0")/ahah
git submodule init
git submodule update
if [ ${1} ]; then
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
                out=`(sed 's/^$/ /g' $f; echo; echo; cat $d/$fbase.in) | $AHEUI`
            else
                #out=`$AHEUI $f`
                out=`sed 's/^$/ /g' $f | $AHEUI`
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
