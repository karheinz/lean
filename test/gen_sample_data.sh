#!/usr/bin/env bash

set +x

# Example:
#
#└── project7
#    ├── 000U_feature1.yaml     => [U]PCOMMING
#    ├── 000T_feature6.yaml     => [T]ODO
#    ├── 020P_feature5.yaml     => IN [P]ROCESS 20%
#    ├── 050P_feature2.yaml     => IN [P]ROCESS 20%
#    ├── 080P_feature7.yaml     => IN [P]ROCESS 80%
#    ├── 100V_feature3.yaml     => IN [V]ERIFICATION 100%
#    ├── WLY_task1.yaml         => [W]EEK[LY] TASK
#    ├── X201909D_feature4.yaml  => [X]VER [D]ONE 2019-09
#    └── X201909C_feature8.yaml  => [X]VER [C]ANCELLED 2019-09

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
        echo "100V_"
    else
        V=$(( $RANDOM % 4 ))
        if (( $V == 0 )); then
            echo DLY_
        elif (( $V == 1 )); then
            echo WLY_
        elif (( $V == 2 )); then
            echo MLY_
        else
            V=$(( $RANDOM % 4 ))
            if (( $V != 0 )); then
                echo "X20190$(( ( $RANDOM % 9 ) + 1 ))D_"
            else
                echo "X20190$(( ( $RANDOM % 9 ) + 1 ))C_"
            fi
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
    FEATURES=$(( $RANDOM % 30 + 1 ))
    for (( K=1; $K <= $FEATURES; ++K )); do
        NAME=feature
        STATE=$(calc_state)
        if [[ $STATE =~ ^[DWM] ]]; then
            NAME=task
        fi

        touch ${DIR}/sample_data/tasks/project${I}/$(calc_state)${NAME}${K}.yaml
    done
done

# Load
pushd ${DIR}/sample_data/
for F in $(find ./people -type f); do
    PREFIX=$(calc_prefix $F people)
    L=${F/*\/people/}
    D=loads/${L%.yaml}
    if [[ -n "$D" ]]; then mkdir -p $D; fi
    for F2 in $(find ./tasks -not -name "X2019*" -type f); do
        PREFIX2=$(calc_prefix $F2 tasks)
        if (( $RANDOM % 20 == 0 )); then
            L2=${D}/${F2/*\/tasks/}
            D2=${L2%/*}
            if [[ -n "$D2" ]]; then mkdir -p $D2; fi

            ln -s ${PREFIX}${PREFIX2}${F2} ${L2}
        fi
    done
done

# Record
for F in $(find ./people -type f); do
    PREFIX=$(calc_prefix $F people)
    L=${F/*\/people/}
    D=track_records/${L%.yaml}
    if [[ -n "$D" ]]; then mkdir -p $D; fi
    for F2 in $(find ./tasks -name "X2019*" -type f); do
        PREFIX2=$(calc_prefix $F2 tasks)
        if true; then
            L2=${D}/${F2/*\/tasks/}
            D2=${L2%/*}
            if [[ -n "$D2" ]]; then mkdir -p $D2; fi

            ln -s ${PREFIX}${PREFIX2}${F2} ${L2}
        fi
    done
done
popd

# Create a view
VIEWS=${DIR}/sample_data/views
QUARTER=${VIEWS}/quarter
mkdir -p $QUARTER
find $VIEWS -type l -delete
pushd ${QUARTER}

# People (not needed, see loads/track_records)
#for F in $(find ../../people -type f); do
#    PREFIX=$(calc_prefix $F people)
#    if (( $RANDOM % 3 != 0 )); then
#        L=${F/*\/..\//}
#        D=${L%/*}
#        if [[ -n "$D" ]]; then mkdir -p $D; fi
#
#        ln -s ${PREFIX}${F} ${L}
#    fi
#done

# Tasks
for F in $(find ../../tasks -type f); do
    PREFIX=$(calc_prefix $F tasks)
    if (( $RANDOM % 3 == 0 )); then
        L=${F/*\/..\//}
        D=${L%/*}
        if [[ -n "$D" ]]; then mkdir -p $D; fi

        ln -s ${PREFIX}${F} ${L}
    fi
done

# Load
for F in $(find ../../people -type f); do
    PREFIX=$(calc_prefix $F people)
    L=${F/*\/..\/people/}
    D=loads${L%.yaml}
    if [[ -n "$D" ]]; then mkdir -p $D; fi
    for F2 in $(find ../../tasks -not -name "X2019*" -type f); do
        PREFIX2=$(calc_prefix $F2 tasks)
        if (( $RANDOM % 20 == 0 )); then
            L2=${D}${F2/*\/..\/tasks/}
            D2=${L2%/*}
            if [[ -n "$D2" ]]; then mkdir -p $D2; fi

            ln -s ${PREFIX}${PREFIX2}${F2} ${L2}
        fi
    done
done

# Record
for F in $(find ../../people -type f); do
    PREFIX=$(calc_prefix $F people)
    L=${F/*\/..\/people/}
    D=track_records${L%.yaml}
    if [[ -n "$D" ]]; then mkdir -p $D; fi
    for F2 in $(find ../../tasks -name "X2019*" -type f); do
        PREFIX2=$(calc_prefix $F2 tasks)
        if true; then
            L2=${D}${F2/*\/..\/tasks/}
            D2=${L2%/*}
            if [[ -n "$D2" ]]; then mkdir -p $D2; fi

            ln -s ${PREFIX}${PREFIX2}${F2} ${L2}
        fi
    done
done

popd
