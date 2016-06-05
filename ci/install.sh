set -ex

. $(dirname $0)/utils.sh

main() {
    case $TARGET in
        cortex-m7*)
            curl -sL https://launchpad.net/gcc-arm-embedded/5.0/5-2016-q1-update/+download/gcc-arm-none-eabi-5_3-2016q1-20160330-linux.tar.bz2 | \
                sudo tar --strip-components 1 -C /usr/local -xj
            sudo apt-get install -y --force-yes --no-install-recommends libc6-i386
            ;;
        cortex-m* | no-linker-field | thumbv*)
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
