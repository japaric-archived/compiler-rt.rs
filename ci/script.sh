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
        cortex-m*|thumbv*)
            local staticlib=$(find -name libcompiler-rt.a)
            local symbols=$(arm-none-eabi-nm -Cg --defined-only $staticlib | grep '^[0-9]' | cut -d' ' -f3)

            for symbol in $symbols; do
                echo $symbol
            done
            echo "Total: $(echo $symbols | wc -w) symbols"

            arm-none-eabi-readelf -A $staticlib
        ;;
    esac
}

main
