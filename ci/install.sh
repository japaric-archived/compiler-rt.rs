if [ $TARGET = thumbv7m-none-eabi ]; then
    sudo apt-get install -y --force-yes --no-install-recommends \
         gcc-arm-none-eabi libnewlib-dev
fi
