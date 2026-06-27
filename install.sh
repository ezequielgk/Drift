#!/usr/bin/env bash
# drift install script
# Installs drift binaries and shell completions for the active shell.

set -euo pipefail

REPO="ezequielgk/drift"
BIN_DIR="${HOME}/.local/bin"
ARCH="x86_64-unknown-linux-gnu"

# ── helpers ───────────────────────────────────────────────────────────────────

info()    { printf '\033[1;34m::\033[0m %s\n' "$*"; }
success() { printf '\033[1;32m✓\033[0m %s\n' "$*"; }
error()   { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

require() {
    command -v "$1" &>/dev/null || error "'$1' is required but not found."
}

# ── detect active shell ───────────────────────────────────────────────────────

detect_shell() {
    local shell
    shell="$(basename "${SHELL:-}")"
    case "${shell}" in
        bash) echo "bash" ;;
        zsh)  echo "zsh"  ;;
        fish) echo "fish" ;;
        *)    echo "unknown" ;;
    esac
}

# ── completion paths ──────────────────────────────────────────────────────────

completion_dir() {
    local shell="$1"
    case "${shell}" in
        bash) echo "${HOME}/.local/share/bash-completion/completions" ;;
        zsh)  echo "${HOME}/.local/share/zsh/site-functions" ;;
        fish) echo "${HOME}/.config/fish/completions" ;;
    esac
}

# ── fetch latest release tag ──────────────────────────────────────────────────

latest_tag() {
    require curl
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
}

# ── main ──────────────────────────────────────────────────────────────────────

main() {
    require curl
    require tar

    local tag
    tag="${1:-$(latest_tag)}"
    [ -n "${tag}" ] || error "Could not determine the latest release tag."

    local pkg="drift-${tag}-${ARCH}"
    local url="https://github.com/${REPO}/releases/download/${tag}/${pkg}.tar.gz"
    local tmp
    tmp="$(mktemp -d)"
    trap "rm -rf '${tmp}'" EXIT

    info "Downloading drift ${tag}..."
    curl -fsSL "${url}" -o "${tmp}/${pkg}.tar.gz" \
        || error "Failed to download ${url}"

    info "Extracting..."
    tar -xzf "${tmp}/${pkg}.tar.gz" -C "${tmp}"

    # ── install binaries ──────────────────────────────────────────────────────
    info "Installing binaries to ${BIN_DIR}..."
    mkdir -p "${BIN_DIR}"
    install -m755 "${tmp}/${pkg}/drift"  "${BIN_DIR}/drift"
    install -m755 "${tmp}/${pkg}/driftd" "${BIN_DIR}/driftd"

    success "drift  → ${BIN_DIR}/drift"
    success "driftd → ${BIN_DIR}/driftd"

    # ── install completions ───────────────────────────────────────────────────
    local shell
    shell="$(detect_shell)"

    if [ "${shell}" = "unknown" ]; then
        info "Shell not detected — skipping completion install."
        info "You can install completions manually from: ${tmp}/${pkg}/completions/"
    else
        local comp_dir
        comp_dir="$(completion_dir "${shell}")"
        mkdir -p "${comp_dir}"

        info "Installing ${shell} completions to ${comp_dir}..."
        case "${shell}" in
            bash)
                install -m644 "${tmp}/${pkg}/completions/drift.bash"  "${comp_dir}/drift"
                install -m644 "${tmp}/${pkg}/completions/driftd.bash" "${comp_dir}/driftd"
                ;;
            zsh)
                install -m644 "${tmp}/${pkg}/completions/_drift"  "${comp_dir}/_drift"
                install -m644 "${tmp}/${pkg}/completions/_driftd" "${comp_dir}/_driftd"
                ;;
            fish)
                install -m644 "${tmp}/${pkg}/completions/drift.fish"  "${comp_dir}/drift.fish"
                install -m644 "${tmp}/${pkg}/completions/driftd.fish" "${comp_dir}/driftd.fish"
                ;;
        esac
        success "Completions installed for ${shell}."
    fi

    # ── PATH reminder ─────────────────────────────────────────────────────────
    if ! echo "${PATH}" | grep -q "${BIN_DIR}"; then
        printf '\n\033[1;33mwarning:\033[0m %s is not in your PATH.\n' "${BIN_DIR}"
        printf 'Add the following to your shell profile:\n\n'
        printf '  export PATH="%s:$PATH"\n\n' "${BIN_DIR}"
    fi

    printf '\n'
    success "drift ${tag} installed successfully."
    printf '\nQuick start:\n'
    printf '  drift toggle    # activate the scroll layout\n'
    printf '  drift status    # check current state\n'
}

main "$@"
