FROM ubuntu:14.04
MAINTAINER Theodore DeRego "tderego94@gmail.com"

# Update repository stuff
RUN apt-get update

# Install all the things we need
RUN apt-get install -y curl libglew-dev libXrandr-dev libopenal-dev libsndfile-dev libfreetype6-dev libjpeg62-dev make

RUN ln -s /usr/lib/x86_64-linux-gnu/libGLEW.so /usr/lib/libGLEW.so.1.5

# Install rust
RUN curl -s https://static.rust-lang.org/rustup.sh | bash

# Install SFML2.1
RUN curl -s http://www.sfml-dev.org/download/sfml/2.1/SFML-2.1-linux-gcc-64bits.tar.bz2 > SFML-2.1-linux-gcc-64bits.tar.bz2
RUN tar -jxvf SFML-2.1-linux-gcc-64bits.tar.bz2 --strip 1 -C /usr
RUN rm SFML-2.1-linux-gcc-64bits.tar.bz2

# Install CSFML2.1
RUN curl -s http://www.sfml-dev.org/download/csfml/CSFML-2.1-linux-gcc-64bits.tar.bz2 > CSFML-2.1-linux-gcc-64bits.tar.bz2
RUN tar -jxvf CSFML-2.1-linux-gcc-64bits.tar.bz2 --strip 1 -C /usr
RUN rm CSFML-2.1-linux-gcc-64bits.tar.bz2

# Default command
CMD /bin/bash
