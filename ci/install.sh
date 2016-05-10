set -ex

. $(dirname $0)/utils.sh

main() {
    case $TARGET in
        cortex-m* | no-linker-field | thumbv7m-none-eabi)
            sudo apt-get install -y --force-yes --no-install-recommends \
                 gcc-arm-none-eabi libnewlib-dev
            ;;
        x86_64-unknown-linux-gnu)
            ;;
        *)
            die "unhandled target: $TARGET"
            ;;
    esac
}

main
