ci_opt := if env("PRE_COMMIT_HOME", "") != "" { "-ci" } else { "" }

precommit:
    just pc{{ci_opt}}

pc:     absolve code-quality
pc-fix: absolve code-quality-fix
pc-ci:  pc-fix
