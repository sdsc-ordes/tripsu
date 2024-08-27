#!/usr/bin/env bash

# Benchmark runtime and memory usage of tripsu
# Compares the working directory version against a baseline branch (main by default)

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
    GIT_CLONE_PROTECTION_ACTIVE=false \
    git clone \
        --branch "${BASE_BRANCH}" \
        "${BASE_URL}" \
        "${BASE_DIR}" \
    && cd "${BASE_DIR}" \
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
    > "${INPUT}" || rm "${INPUT}"
fi

# setup config
RULES=$(mktemp)
BASE_IDX=$(mktemp)
COMP_IDX=$(mktemp)

cat << EOF > "${RULES}"

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

### functions for profiling

cpu_prof() {
    local branch1=$1
    local cmd1=$2
    local branch2=$3
    local cmd2=$4
    local out=$5
    hyperfine --export-markdown "${out}" -r 5 \
        -n "${branch1}" "${cmd1}" \
        -n "${branch2}" "${cmd2}"
}

mem_prof() {
    local name=$1
    local cmd=$2
    local heap_out
    heap_out=$(mktemp)
    echo -n "$name: "
    # shellcheck disable=SC2086
    heaptrack -o "${heap_out}" ${cmd} >/dev/null
    heaptrack_print "${heap_out}.zst" \
    | grep '^peak heap memory'
}

make_report() {
    local cpu_index=$1
    local cpu_pseudo=$2
    local mem_index=$3
    local mem_pseudo=$4
    local base_branch=$5

    cat <<-MD
	# tripsu profiling

	> date: $(date -u +%Y-%m-%d)

    Comparing $(git branch --show-current) against $base_branch.
	
	## Timings
	
	Run time compared using hyperfine
	
	### Indexing
	
	$(cat "${cpu_index}")
	
	### Pseudonymization
	
	$(cat "${cpu_pseudo}")
	
	## Memory
	
	Heap memory usage compared using heaptrack
	
	### Indexing
	
	$(cat "${mem_index}")
	
	### Pseudonymization
	
	$(cat "${mem_pseudo}")
	MD
}


###  Run profiling

## Profile cpu time
HYPF_IDX_OUT=$(mktemp)
HYPF_PSD_OUT=$(mktemp)

# indexing
cpu_prof "${BASE_BRANCH}" "${BASE_CMD_IDX}" \
         "${COMP_BRANCH}" "${COMP_CMD_IDX}" "${HYPF_IDX_OUT}"
# pseudonymization
cpu_prof "${BASE_BRANCH}" "${BASE_CMD_IDX}" \
         "${COMP_BRANCH}" "${COMP_CMD_IDX}" "${HYPF_PSD_OUT}"

## Profile memory
HEAP_IDX_OUT=$(mktemp)
HEAP_PSD_OUT=$(mktemp)

# indexing
mem_prof "${BASE_BRANCH}" "${BASE_CMD_IDX}" >  "${HEAP_IDX_OUT}"
mem_prof "${COMP_BRANCH}" "${COMP_CMD_IDX}" >> "${HEAP_IDX_OUT}"
# pseudonymization
mem_prof "${BASE_BRANCH}" "${BASE_CMD_PSD}" >  "${HEAP_PSD_OUT}"
mem_prof "${COMP_BRANCH}" "${COMP_CMD_PSD}" >> "${HEAP_PSD_OUT}"


### Reporting
make_report \
    "${HYPF_IDX_OUT}" "${HYPF_PSD_OUT}" \
    "${HEAP_IDX_OUT}" "${HEAP_PSD_OUT}" \
    "${BASE_BRANCH}" > "${OUTPUT}"
