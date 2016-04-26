set -ex

. $(dirname $0)/utils.sh

main() {
    local prefix=
    case $TARGET in
        thumbv7m-none-eabi)
            prefix=arm-none-eabi-
            ;;
        x86_64-unknown-linux-gnu)
            prefix=
            ;;
        *)
            die "unhandled target: $TARGET"
            ;;
    esac

    export AR_${TARGET//-/_}=${prefix}ar
    export CC_${TARGET//-/_}=${prefix}gcc

    cargo build --target $TARGET
}

main
