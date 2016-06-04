set -ex

. $(dirname $0)/utils.sh

main() {
    case $TARGET in
        no-linker-field)
            export AR_no_linker_field=arm-none-eabi-ar
            export CC_no_linker_field=arm-none-eabi-gcc
            ;;
    esac

    cargo build --target $TARGET

    case $TARGET in
        cortex-m*|thumb*)
            arm-none-eabi-readelf -h $(find -name libcompiler-rt.a)
        ;;
    esac
}

main
