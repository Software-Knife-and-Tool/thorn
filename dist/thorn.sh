#!/usr/bin/env bash
BASE=${THORN_HOME:=/opt/thorn}
BASE_PRELUDE=$BASE/thorn/prelude

usage () {
    echo "usage: $0 [session options] [runtime options] src-file..." >&2
    echo "[session options]" >&2
    echo "  -h, --help                 print this message and exit." >&2
    echo "  --config=config-list       runtime configuration." >&2
    echo "[runtime options]" >&2
    echo "  --eval=form                evaluate form and print result." >&2
    echo "  --quiet-eval=form          evaluate form quietly." >&2
    echo "  --load=src-file            load src-file in sequence." >&2
    echo "  --pipe                     run in pipe mode." >&2
    echo "  --version                  print version and exit." >&2
    exit 2
}

PRELUDE_FILES=""
OPTIONS=""
SOURCES=""

# prelude.l needs to be first
PRELUDE=(	       	\
      prelude.l		\
      backquote.l	\
      boole.l		\
      break.l		\
      compile.l		\
      ctype.l		\
      describe.l       	\
      environment.l	\
      exception.l	\
      fasl.l		\
      fixnum.l		\
      format.l		\
      funcall.l		\
      function.l	\
      inspect.l		\
      lambda.l		\
      list.l		\
      log.l		\
      macro.l		\
      map.l		\
      namespace.l	\
      parse.l		\
      read-macro.l	\
      read.l		\
      repl.l		\
      stream.l		\
      string.l		\
      symbol-macro.l	\
      symbol.l		\
      time.l		\
      type.l		\
      vector.l		\
    )

for src in ${PRELUDE[@]}; do
    PRELUDE_FILES+=" -l $BASE_PRELUDE/$src"
done

optspec=":h-:"
while getopts "$optspec" optchar; do
    case "${optchar}" in
        -)
            case "${OPTARG}" in
                eval=*)
                    val=${OPTARG#*=}
                    opt=${OPTARG%=$val}
                    OPTIONS+=" -e \"${val}\""
                    ;;
                load=*)
                    val=${OPTARG#*=}
                    opt=${OPTARG%=$val}
                    OPTIONS+=" -l \"${val}\""
                    ;;
                quiet-eval=*)
                    val=${OPTARG#*=}
                    opt=${OPTARG%=$val}
                    OPTIONS+=" -q \"${val}\""
                    ;;
                config=*)
                    val=${OPTARG#*=}
                    opt=${OPTARG%=$val}
                    OPTIONS+=" -c \"${val}\""
                    ;;
                help)
                    usage
                    ;;
                version)
                    $BASE/bin/mu-shell -v
                    echo
                    exit 2
                    ;;
                pipe)
                    OPTIONS+=" -p"
                    ;;
                *)
                    if [ "$OPTERR" = 1 ] && [ "${optspec:0:1}" != ":" ]; then
                        echo "Unknown option --${OPTARG}" >&2
                    fi
                    ;;
            esac;;
        h)
            usage
            ;;
        *)
            if [ "$OPTERR" != 1 ] || [ "${optspec:0:1}" = ":" ]; then
                echo "Non-option argument: '-${OPTARG}'" >&2
            fi
            ;;
    esac
done

len="${#@}"


for (( i=${OPTIND}; i<="${#@}"; i++ )); do SOURCES+=" \"${!i}\"" ; done

export THORN_LOAD_LIST=SOURCES
eval $BASE/bin/mu-shell $PRELUDE_FILES -q "\(prelude:%init-ns\)" $OPTIONS # $BASE/thorn/thorn.l ${SOURCES[@]}
