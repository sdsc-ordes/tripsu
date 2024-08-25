#!/usr/bin/env bash

set -euo pipefail

### Final output path
OUTPUT="profiling.md"
PROFILE='release'
BUILD_ARGS=( )
[[ "${PROFILE}" == 'release' ]] && BUILD_ARGS+=( '--release' )
### setup binaries

# baseline binary
BASE_BRANCH='main'

BASE_DIR=$(mktemp -d)
BASE_URL="$(git config --get remote.origin.url)"
(
    git clone \
        --branch ${BASE_BRANCH} \
        ${BASE_URL} \
        ${BASE_DIR} \
    && cd ${BASE_DIR} \
    && just build "${BUILD_ARGS[@]}"
)
BASE_BIN="${BASE_DIR}/target/${PROFILE}/tripsu"

# current binary
COMP_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
just build "${BUILD_ARGS[@]}"
COMP_BIN="./target/${PROFILE}/tripsu"

# setup data
DATA_URL="https://ftp.uniprot.org/pub/databases/uniprot/current_release/rdf/proteomes.rdf.xz"
INPUT="/tmp/proteomes.nt"

# Download data if needed
if [ ! -f ${INPUT} ]; then
    curl "${DATA_URL}" \
    | xz -dc -  \
    | rdfpipe-rs -i rdf-xml -o nt - \
    > ${INPUT} || rm ${INPUT}
fi

# setup config
RULES=$(mktemp)
BASE_IDX=$(mktemp)
COMP_IDX=$(mktemp)

cat << EOF > ${RULES}

nodes:
  of_type:
    - "http://purl.uniprot.org/core/Proteome"
    - "http://purl.uniprot.org/core/Strain"

objects:
  on_type_predicate:
    "http://purl.uniprot.org/core/Submission_Citation":
      - "http://purl.uniprot.org/core/author"
  
  on_predicate:
    - "http://purl.org/dc/terms/identifier"

EOF

### Commands to benchmark

BASE_CMD_IDX="${BASE_BIN} index -o ${BASE_IDX} ${INPUT}"
COMP_CMD_IDX="${COMP_BIN} index -o ${COMP_IDX} ${INPUT}"
BASE_CMD_PSD="${BASE_BIN} pseudo -r ${RULES} -x ${BASE_IDX} ${INPUT}"
COMP_CMD_PSD="${COMP_BIN} pseudo -r ${RULES} -x ${COMP_IDX} ${INPUT}"

### Profile time

HYPF_OPTS=( -r 2 )
HYPF_IDX_OUT=$(mktemp)
HYPF_PSD_OUT=$(mktemp)

# indexing
hyperfine --export-markdown "${HYPF_IDX_OUT}" "${HYPF_OPTS[@]}" \
    -n "${BASE_BRANCH}" "${BASE_CMD_IDX}" \
    -n "${COMP_BRANCH}" "${COMP_CMD_IDX}"
# pseudonymization
hyperfine --export-markdown "${HYPF_PSD_OUT}" "${HYPF_OPTS[@]}" \
    -n "${BASE_BRANCH}" "${BASE_CMD_PSD}" \
    -n "${COMP_BRANCH}" "${COMP_CMD_PSD}"


### Profile memory

HEAP_IDX_OUT=$(mktemp)
HEAP_PSD_OUT=$(mktemp)

mem_prof() {
    local name=$1
    local cmd=$2
    local HEAP_OUT=$(mktemp)
    echo -n "$name: "
    heaptrack -o "${HEAP_OUT}" ${cmd} >/dev/null
    heaptrack_print "${HEAP_OUT}.zst" \
    | grep '^peak heap memory'
}

# indexing
mem_prof "${BASE_BRANCH}" "${BASE_CMD_IDX}" >  ${HEAP_IDX_OUT}
mem_prof "${COMP_BRANCH}" "${COMP_CMD_IDX}" >> ${HEAP_IDX_OUT}
# pseudonymization
mem_prof "${BASE_BRANCH}" "${BASE_CMD_PSD}" >  ${HEAP_PSD_OUT}
mem_prof "${COMP_BRANCH}" "${COMP_CMD_PSD}" >> ${HEAP_PSD_OUT}

### Reporting

cat << MD > ${OUTPUT}
# tripsu profiling

> date: $(date -u +%Y-%m-%d)

Comparing branch XXX against YYY.

## Timings

Run time compared using hyperfine

### Indexing

$(cat ${HYPF_IDX_OUT})

### Pseudonymization

$(cat ${HYPF_PSD_OUT})

## Memory

Heap memory usage compared using heaptrack

### Indexing

$(cat ${HEAP_IDX_OUT})

### Pseudonymization

$(cat ${HEAP_PSD_OUT})
MD
