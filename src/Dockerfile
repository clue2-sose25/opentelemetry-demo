# Ad service
FROM --platform=${BUILDPLATFORM} eclipse-temurin:21-jdk AS adbuilder
ARG _JAVA_OPTIONS
WORKDIR /usr/src/app/

COPY ./src/ad/gradlew* ./src/ad/settings.gradle* ./src/ad/build.gradle ./
COPY ./src/ad/gradle ./gradle

RUN chmod +x ./gradlew
RUN ./gradlew
RUN ./gradlew downloadRepos

COPY ./src/ad/ ./
COPY ./pb/ ./proto
RUN chmod +x ./gradlew
RUN ./gradlew installDist -PprotoSourceDir=./proto

# Cart service
FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:8.0 AS cartbuilder
ARG TARGETARCH

WORKDIR /usr/src/app/

COPY ./src/cart/ ./
COPY ./pb/ ./pb/

RUN dotnet restore ./src/cart.csproj -r linux-musl-$TARGETARCH

RUN dotnet publish ./src/cart.csproj -r linux-musl-$TARGETARCH --no-restore -o /cart

# Checkout service
FROM golang:1.24.6-alpine AS checkoutbuilder

WORKDIR /usr/src/app/

COPY ./src/checkout/go.mod go.mod
COPY ./src/checkout/go.sum go.sum

RUN go mod download

COPY ./src/checkout/genproto/oteldemo/ genproto/oteldemo/
COPY ./src/checkout/kafka/ kafka/
COPY ./src/checkout/money/ money/
COPY ./src/checkout/main.go main.go

RUN CGO_ENABLED=0 GOOS=linux go build -ldflags "-s -w" -o checkout main.go

# Currency service
FROM php:8.3-cli-alpine3.18 AS currencybilder
ARG OPENTELEMETRY_CPP_VERSION="1.20.0"
RUN apk update && apk add git cmake make g++ grpc-dev protobuf-dev linux-headers

RUN git clone --depth 1 --branch v${OPENTELEMETRY_CPP_VERSION} https://github.com/open-telemetry/opentelemetry-cpp \
    && cd opentelemetry-cpp/ \
    && mkdir build \
    && cd build \
    && cmake .. -DCMAKE_CXX_STANDARD=17 -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
          -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF \
          -DWITH_EXAMPLES=OFF -DWITH_OTLP_GRPC=ON -DWITH_ABSEIL=ON \
    && make -j$(nproc || sysctl -n hw.ncpu || echo 1) install && cd ../..

COPY ./src/currency /currency
COPY ./pb/demo.proto /currency/proto/demo.proto

RUN cd /currency \
    && mkdir -p build && cd build \
    && cmake .. \
    && make -j$(nproc || sysctl -n hw.ncpu || echo 1) install


# Frontend
FROM node:22-alpine AS frontendbuilder
RUN apk add --no-cache libc6-compat

USER node

WORKDIR /app

COPY ./src/frontend/package*.json ./
RUN npm clean-install --no-audit --no-progress
COPY ./src/frontend/protos ./protos
COPY ./src/frontend .

RUN npm run build

FROM node:22-alpine AS frontenddeps

USER node

WORKDIR /app
COPY --chown=node:node --from=frontendbuilder /home/node/.npm/ /home/node/.npm/
COPY ./src/frontend/package*.json ./
RUN NODE_ENV=production npm clean-install --prefer-offline --no-audit --no-progress

# Payment service
FROM node:22-alpine AS paymentbuild

WORKDIR /usr/src/app/

COPY ./src/payment/package*.json ./

RUN apk add --no-cache python3 make g++ && npm ci --omit=dev

# Product Catalog service
FROM golang:1.24.6-alpine AS catalogbuilder

WORKDIR /usr/src/app/

COPY ./src/product-catalog/go.mod go.mod
COPY ./src/product-catalog/go.sum go.sum

RUN go mod download

COPY ./src/product-catalog/genproto/oteldemo/ genproto/oteldemo/
COPY ./src/product-catalog/products/ products/
COPY ./src/product-catalog/main.go main.go

RUN CGO_ENABLED=0 GOOS=linux GO111MODULE=on go build -ldflags "-s -w" -o product-catalog main.go

FROM composer:2.8 AS vendor

WORKDIR /tmp/
COPY ./src/quote/composer.json .

RUN composer install \
    --ignore-platform-reqs \
    --no-interaction \
    --no-plugins \
    --no-scripts \
    --no-dev \
    --prefer-dist

#Recommendation Service
FROM python:3.13-alpine AS recommendation-builder
RUN apk update --no-cache \
    && apk add --no-cache build-base g++\
    && rm -rf /var/cache/apk/*

WORKDIR /usr/src/app/
COPY ./src/recommendation/requirements.txt ./

RUN pip install --upgrade pip
RUN pip install --prefix="/reqs" -r requirements.txt

# Shipping service
FROM --platform=${BUILDPLATFORM} rust:1.88-alpine AS shipping-builder
ARG TARGETARCH
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN echo Building on ${BUILDPLATFORM} for ${TARGETPLATFORM}

# Check if we are doing cross-compilation, if so we need to add in some more dependencies and run rustup
RUN if [ "${TARGETPLATFORM}" = "${BUILDPLATFORM}" ] ; then \
        apk update && apk add --no-cache g++ musl-dev protobuf-dev protobuf-c-dev ca-certificates wget; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] ; then \
        apk update && apk add --no-cache gcc-aarch64-none-linux-gnu g++-aarch64-none-linux-gnu musl-dev protobuf-dev protobuf-c-dev ca-certificates wget && \
        rustup target add aarch64-unknown-linux-gnu && \
        rustup toolchain install stable-aarch64-unknown-linux-gnu; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ] ; then \
        apk update && apk add --no-cache gcc-x86_64-linux-gnu g++-x86_64-linux-gnu musl-dev protobuf-dev protobuf-c-dev ca-certificates wget && \
        rustup target add x86_64-unknown-linux-gnu && \
        rustup toolchain install stable-x86_64-unknown-linux-gnu; \
    else \
        echo "${TARGETPLATFORM} is not supported"; \
        exit 1; \
    fi

WORKDIR /app/

COPY /src/shipping/ /app/
COPY /pb/ /app/proto/

# Compile or crosscompile
RUN if [ "${TARGETPLATFORM}" = "${BUILDPLATFORM}" ] ; then \
        cargo build -r --features="dockerproto"; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] ; then \
        env CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
            CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
            CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
        cargo build -r --features="dockerproto" --target aarch64-unknown-linux-gnu && \
        cp /app/target/aarch64-unknown-linux-gnu/release/shipping /app/target/release/shipping; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ] ; then \
        env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
            CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
            CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ \
        cargo build -r --features="dockerproto" --target x86_64-unknown-linux-gnu && \
        cp /app/target/x86_64-unknown-linux-gnu/release/shipping /app/target/release/shipping; \
    else \
        echo "${TARGETPLATFORM} is not supported"; \
        exit 1; \
    fi


ENV GRPC_HEALTH_PROBE_VERSION=v0.4.37
RUN wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-${TARGETARCH} && \
    chmod +x /bin/grpc_health_probe

FROM php:8.3-cli-alpine3.18 AS runner

ARG OTEL_JAVA_AGENT_VERSION="2.15.0"
ARG _JAVA_OPTIONS

# Installing Java 21 JRE (because in Alpine 3.18 it is not available, but we need this for currency)
ARG TARGETARCH
ENV JAVA_VERSION="21.0.8+9" \
    JAVA_HOME=/opt/java/openjdk \
    PATH="$PATH:/opt/java/openjdk/bin"

# Install curl and ca-certificates
RUN apk add --no-cache curl ca-certificates

# Download and extract Temurin OpenJDK 21 JRE for the appropriate architecture
RUN case "${TARGETARCH}" in \
        "amd64")  ARCH="x64" ;; \
        "arm64")  ARCH="aarch64" ;; \
        *)        echo "Unsupported arch: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    VERSION="$(echo $JAVA_VERSION | cut -d'+' -f1)" && \
    BUILD="$(echo $JAVA_VERSION | cut -d'+' -f2)" && \
    ENCODED_VERSION="$(echo $JAVA_VERSION | sed 's/+/%2B/')" && \
    FILENAME_VERSION="${VERSION}_${BUILD}" && \
    JDK_VERSION="$(echo $JAVA_VERSION | tr '+' '-')" && \
    echo "Downloading OpenJDK 21 JRE for ${ARCH}...https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${ENCODED_VERSION}/OpenJDK21U-jre_${ARCH}_alpine-linux_hotspot_${FILENAME_VERSION}.tar.gz" && \
    curl -fsSL "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${ENCODED_VERSION}/OpenJDK21U-jre_${ARCH}_alpine-linux_hotspot_${FILENAME_VERSION}.tar.gz" -o /tmp/jre.tar.gz && \
    mkdir -p "$JAVA_HOME" && \
    tar -xzf /tmp/jre.tar.gz -C "$JAVA_HOME" --strip-components=1 && \
    rm -rf /tmp/jre.tar.gz

RUN apk update && apk add \
    wget curl ca-certificates \
    dotnet7-runtime \
    ruby ruby-dev ruby-bundler build-base libffi-dev  protobuf-dev \
    nodejs npm \
    # php83-cli php83-common php83-opcache php83-pcntl php83-curl php83-mbstring php83-xml php83-bcmath php83-session php83-json php83-dev php83-pear \
    python3 py3-pip python3-dev \
    grpc protobuf \
    g++ grpc-dev protobuf-dev linux-headers \
    supervisor && \
    ln -sf /usr/bin/python3 /usr/bin/python && \
    ln -sf /usr/bin/php83 /usr/bin/php

ADD --chmod=0755 https://github.com/mlocati/docker-php-extension-installer/releases/latest/download/install-php-extensions /usr/local/bin/

RUN install-php-extensions protobuf opentelemetry opcache pcntl


RUN if [ "${TARGETPLATFORM}" = "${BUILDPLATFORM}" ] ; then \
        apk update && apk add --no-cache g++ musl-dev protobuf-dev protobuf-c-dev ca-certificates wget; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] ; then \
        apk update && apk add --no-cache gcc-aarch64-none-linux-gnu g++-aarch64-none-linux-gnu musl-dev protobuf-dev protobuf-c-dev ca-certificates wget && \
        rustup target add aarch64-unknown-linux-gnu && \
        rustup toolchain install stable-aarch64-unknown-linux-gnu; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ] ; then \
        apk update && apk add --no-cache gcc-x86_64-linux-gnu g++-x86_64-linux-gnu musl-dev protobuf-dev protobuf-c-dev ca-certificates wget && \
        rustup target add x86_64-unknown-linux-gnu && \
        rustup toolchain install stable-x86_64-unknown-linux-gnu; \
    else \
        echo "${TARGETPLATFORM} is not supported"; \
        exit 1; \
    fi

RUN addgroup -g 1001 nodejs && \
    adduser -D -u 1001 -G nodejs -s /bin/sh nextjs

RUN mkdir /ads && mkdir /cart && mkdir /currency && mkdir /checkout && mkdir /frontend && mkdir /payment && mkdir /catalog && mkdir /shipping && mkdir /recom && mkdir email_server
COPY --from=adbuilder /usr/src/app/ /ads
ADD https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/v$OTEL_JAVA_AGENT_VERSION/opentelemetry-javaagent.jar /ads/opentelemetry-javaagent.jar
ENV JAVA_TOOL_OPTIONS=-javaagent:/ads/opentelemetry-javaagent.jar

ENV DOTNET_HOSTBUILDER__RELOADCONFIGONCHANGE=false
COPY --from=cartbuilder /cart /cart
COPY --from=checkoutbuilder /usr/src/app/checkout /checkout
COPY --from=currencybilder /usr/local /usr/local

COPY ./src/email/ /email_server
WORKDIR /email_server
RUN rm /email_server/Gemfile.lock && bundle config set force_ruby_platform true && bundle install

COPY --from=frontendbuilder --chown=nextjs:nodejs /app/next.config.js /frontend
COPY --from=frontendbuilder --chown=nextjs:nodejs /app/utils/telemetry/Instrumentation.js /frontend
COPY --from=frontendbuilder --chown=nextjs:nodejs /app/public /frontend/public
COPY --from=frontendbuilder --chown=nextjs:nodejs /app/package.json /frontend/package.json
COPY --from=frontenddeps --chown=nextjs:nodejs /app/node_modules /frontend/node_modules
COPY --from=frontendbuilder --chown=nextjs:nodejs /app/.next/standalone /frontend/
COPY --from=frontendbuilder --chown=nextjs:nodejs /app/.next/static /frontend/.next/static

COPY --chown=nextjs:nextjs --from=paymentbuild /usr/src/app/node_modules/ /payment/node_modules/
COPY ./src/payment/ /payment
COPY ./pb/demo.proto /payment

COPY ./src/product-catalog/products/ /catalog/products/
COPY --from=catalogbuilder /usr/src/app/product-catalog/ /catalog

WORKDIR /recom
COPY ./src/recommendation/ /recom
RUN pip install --break-system-packages \
    "opentelemetry-distro==0.51b0" \
    "opentelemetry-exporter-otlp-proto-grpc==1.30.0" \
    "opentelemetry-instrumentation==0.51b0" \
    "opentelemetry-instrumentation-requests==0.51b0" \
    "opentelemetry-instrumentation-urllib3==0.51b0" \
    "opentelemetry-instrumentation-urllib==0.51b0" \
    grpcio grpcio-health-checking && \
    opentelemetry-bootstrap -a install && \
    pip install --break-system-packages -r /recom/requirements.txt

COPY --from=shipping-builder /app/target/release/shipping /shipping/shipping
COPY --from=shipping-builder /bin/grpc_health_probe /bin/grpc_health_probe


EXPOSE ${QUOTE_PORT}
COPY --from=vendor --chown=:www-data /tmp/vendor/ /var/www/vendor/
COPY ./src/quote/ /var/www

COPY ./src/mono/supervisord.conf /etc/supervisord.conf
USER root
WORKDIR /
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]