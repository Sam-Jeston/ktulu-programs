# LP Aggregator

## TODOs
 - Name project and populate README
 - Remove all `UncheckedAccount` types
 - Ensure Vault and associated token accounts are closeable

# Misc

## Proc Macro2 IDL issue

Workaround: https://stackoverflow.com/questions/79582055/why-is-the-method-source-file-not-found-for-proc-macro2span

TLDR; `anchor build --no-idl` to build program, then use nightly toolchain to build IDLs when required


## Sync program ids
`anchor keys sync`