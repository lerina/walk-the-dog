if [[ $# -eq 0 ]] ; then
    echo 'clean what?'
    exit 1
fi

rm -fr $1*
ls
