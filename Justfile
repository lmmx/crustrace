# Just is a task runner, like Make but without the build system / dependency tracking part.
# docs: https://github.com/casey/just
#
# The `-ci` variants are ran in CI, they do command grouping on GitHub Actions, set consistent env vars etc.,
# but they require bash.
#
# The non`-ci` variants can be run locally without having bash installed.

set dotenv-load

default: precommit prepush

precommit: code-quality absolve
prepush: clippy test
precommit-fix: code-quality-fix

commit-msg message:
  printf "{{ message }}" | conventional_commits_linter --from-stdin --allow-angular-type-only

ci: precommit prepush docs

clippy-all:
    cargo clippy --workspace --all-targets --all-features --target-dir target/clippy-all-features -- -D warnings

clippy:
    cargo clippy --workspace --all-targets --target-dir target/clippy -- -D warnings

test *args:
    cargo nextest run {{args}} < /dev/null

example-instrument:
    cargo run --bin instrument_fib

example-omni:
    cargo run --bin omni_fib

test-ci *args:
    #!/usr/bin/env -S bash -euo pipefail
    source .envrc
    echo -e "\033[1;33m🏃 Running all but doc-tests with nextest...\033[0m"
    cmd_group "cargo nextest run --features slow-tests {{args}} < /dev/null"

    echo -e "\033[1;36m📚 Running documentation tests...\033[0m"
    cmd_group "cargo test --features slow-tests --doc {{args}}"

doc-tests *args:
    cargo test --doc {{args}}

doc-tests-ci *args:
    #!/usr/bin/env -S bash -euo pipefail
    source .envrc
    echo -e "\033[1;36m📚 Running documentation tests...\033[0m"
    cmd_group "cargo test --doc {{args}}"

fix-eof-ws mode="":
    #!/usr/bin/env sh
    ARGS=''
    if [ "{{mode}}" = "check" ]; then
        ARGS="--check-only"
    fi
    whitespace-format --add-new-line-marker-at-end-of-file \
          --new-line-marker=linux \
          --normalize-new-line-markers \
          --exclude ".git/|target/|.json$|.lock$" \
          $ARGS \
          .

code-quality:
    taplo lint
    taplo format --check
    just fix-eof-ws check
    cargo machete
    cargo fmt --check --all

code-quality-fix:
    taplo lint
    taplo format
    just fix-eof-ws
    cargo machete
    cargo fmt --all

absolve:
    ./crustrace-dev/absolve.sh

ship:
    #!/usr/bin/env -S bash -euo pipefail
    # Refuse to run if not on master branch or not up to date with origin/master
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$branch" != "master" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: not on 'master' branch (current: $branch)\033[0m"
    exit 1
    fi
    git fetch origin master
    local_rev="$(git rev-parse HEAD)"
    remote_rev="$(git rev-parse origin/master)"
    if [[ "$local_rev" != "$remote_rev" ]]; then
    echo -e "\033[1;31m❌ Refusing to run: local master branch is not up to date with origin/master\033[0m"
    echo -e "Local HEAD:  $local_rev"
    echo -e "Origin HEAD: $remote_rev"
    echo -e "Please pull/rebase to update."
    exit 1
    fi
    release-plz update
    git add .
    git commit -m "Upgrades"
    git push
    just publish

publish:
    git_token := $(gh auth token 2>/dev/null) || echo $PUBLISH_GITHUB_TOKEN
    release-plz release --backend github --git-token $git_token

docsrs *args:
    #!/usr/bin/env -S bash -eux
    source .envrc
    export RUSTDOCFLAGS="--cfg docsrs"
    cargo +nightly doc {{args}}

docs:
    cargo doc --workspace --all-features --no-deps --document-private-items --keep-going

lockfile:
    cargo update --workspace --locked

# Test if code will build (compile only, no linking) for a specific target
test-build-target target:
    cargo check --release --target {{target}}

# Test if Windows builds will compile (no linking)
test-windows-build:
    #!/usr/bin/env -S bash -euo pipefail
    echo -e "\033[1;33m🏗️ Testing Windows builds (compile only)...\033[0m"

    echo -e "\033[1;36m📦 Testing x86_64-pc-windows-msvc...\033[0m"
    just test-build-target x86_64-pc-windows-msvc

    echo -e "\033[1;36m📦 Testing aarch64-pc-windows-msvc...\033[0m"
    just test-build-target aarch64-pc-windows-msvc

    echo -e "\033[1;32m✅ Windows builds compilation check completed!\033[0m"

# Test if Apple/macOS builds will compile (no linking)
test-apple-build:
    #!/usr/bin/env -S bash -euo pipefail
    echo -e "\033[1;33m🏗️ Testing Apple/macOS builds (compile only)...\033[0m"

    echo -e "\033[1;36m📦 Testing x86_64-apple-darwin...\033[0m"
    just test-build-target x86_64-apple-darwin

    echo -e "\033[1;36m📦 Testing aarch64-apple-darwin...\033[0m"
    just test-build-target aarch64-apple-darwin

    echo -e "\033[1;32m✅ Apple builds compilation check completed!\033[0m"

# Test if Linux builds will compile (no linking)
test-linux-build:
    #!/usr/bin/env -S bash -euo pipefail
    echo -e "\033[1;33m🏗️ Testing Linux builds (compile only)...\033[0m"

    echo -e "\033[1;36m📦 Testing x86_64-unknown-linux-gnu...\033[0m"
    just test-build-target x86_64-unknown-linux-gnu

    echo -e "\033[1;36m📦 Testing x86_64-unknown-linux-musl...\033[0m"
    just test-build-target x86_64-unknown-linux-musl

    echo -e "\033[1;36m📦 Testing aarch64-unknown-linux-gnu...\033[0m"
    just test-build-target aarch64-unknown-linux-gnu

    echo -e "\033[1;36m📦 Testing aarch64-unknown-linux-musl...\033[0m"
    just test-build-target aarch64-unknown-linux-musl

    echo -e "\033[1;32m✅ Linux builds compilation check completed!\033[0m"

# Test if FreeBSD build will compile (no linking)
test-freebsd-build:
    #!/usr/bin/env -S bash -euo pipefail
    echo -e "\033[1;33m🏗️ Testing FreeBSD build (compile only)...\033[0m"

    echo -e "\033[1;36m📦 Testing x86_64-unknown-freebsd...\033[0m"
    just test-build-target x86_64-unknown-freebsd

    echo -e "\033[1;32m✅ FreeBSD build compilation check completed!\033[0m"

# Test if all targets will compile (no linking)
test-all-builds:
    #!/usr/bin/env -S bash -euo pipefail
    echo -e "\033[1;33m🏗️ Testing all targets (compile only)...\033[0m"

    just test-linux-build
    just test-apple-build
    just test-windows-build
    just test-freebsd-build

    echo -e "\033[1;32m✅ All target builds compilation check completed!\033[0m"
