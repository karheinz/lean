#!/usr/bin/env bash

set -e
shopt -s extglob

function calc_prefix()
{
    local SUB_PATH DEPTH PREFIX
    SUB_PATH=${1##*/$2/}
    DEPTH=$(export IFS=/; for D in $SUB_PATH; do echo; done | wc -l)

    PREFIX=""
    for (( I=0; $I < $DEPTH; ++I )); do
        PREFIX+="../"
    done

    echo $PREFIX
}

function calc_state_suffix()
{
    local STATES=( S )
    if (( $1 == 0 )); then
        STATES[${#STATES[@]}]=U
        STATES[${#STATES[@]}]=U
        STATES[${#STATES[@]}]=T
        STATES[${#STATES[@]}]=T
    else
        STATES[${#STATES[@]}]=P
        STATES[${#STATES[@]}]=P
        STATES[${#STATES[@]}]=P
        STATES[${#STATES[@]}]=P
    fi

    echo ${STATES[$(( $RANDOM % 5 ))]}
}

function calc_state()
{
    local V=$(( ( $RANDOM % 12 ) * 10 ))

    if (( $V < 10 )); then
        echo "000$(calc_state_suffix $V)_"
    elif (( $V < 100 )); then
        echo "0${V}$(calc_state_suffix $V)_"
    elif (( $V == 100 )); then
        V=$(( $RANDOM % 2 ))
        if (( $V == 0 )); then
            echo "100D_"
        else
            echo "100V_"
        fi
    else
        V=$(( $RANDOM % 3 ))
        if (( $V == 0 )); then
            echo DLY_
        elif (( $V == 1 )); then
            echo WLY_
        else
            echo MLY_
        fi
    fi
}

DIR=${0%/*}
rm -rf ${DIR}/sample_data

# Create data
mkdir -p ${DIR}/sample_data/people/external
PEOPLE=$(( $RANDOM % 8 + 5 ))
for (( I=1; $I <= $PEOPLE; ++I )); do
    touch ${DIR}/sample_data/people/person${I}.yaml
done
EXT_PEOPLE=$(( PEOPLE + ( $RANDOM % 3 ) + 2 ))
(( ++PEOPLE ))
for (( I=$PEOPLE; $I <= $EXT_PEOPLE; ++I )); do
    touch ${DIR}/sample_data/people/external/person${I}.yaml
done

PROJECTS=$(( $RANDOM % 5 + 5 ))
for (( I=1; $I <= $PROJECTS; ++I )); do
    mkdir -p ${DIR}/sample_data/tasks/project${I}
    FEATURES=$(( $RANDOM % 20 + 1 ))
    for (( K=1; $K <= $FEATURES; ++K )); do
        touch ${DIR}/sample_data/tasks/project${I}/$(calc_state)feature${K}.yaml
    done
done

# Create a view
VIEWS=${DIR}/sample_data/views
QUARTER=${VIEWS}/quarter
mkdir -p $QUARTER
find $VIEWS -type l -delete
pushd ${QUARTER}
for F in $(find ../../people -type f); do
    PREFIX=$(calc_prefix $F people)
    if (( $RANDOM % 3 != 0 )); then
        L=${F/*\/..\//}
        D=${L%/*}
        if [[ -n "$D" ]]; then mkdir -p $D; fi

        ln -s ${PREFIX}${F} ${L}
    fi
done
for F in $(find ../../tasks -type f); do
    PREFIX=$(calc_prefix $F tasks)
    if (( $RANDOM % 3 == 0 )); then
        L=${F/*\/..\//}
        D=${L%/*}
        if [[ -n "$D" ]]; then mkdir -p $D; fi

        ln -s ${PREFIX}${F} ${L}
    fi
done
popd
