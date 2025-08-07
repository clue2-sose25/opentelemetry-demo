# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0


FROM --platform=${BUILDPLATFORM} ghcr.io/graalvm/graalvm-community:21-ol9 AS builder
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
# Build native image instead of regular distribution
RUN ./gradlew nativeCompile -PprotoSourceDir=./proto

# -----------------------------------------------------------------------------

FROM ghcr.io/graalvm/graalvm-community:21-ol9

ARG OTEL_JAVA_AGENT_VERSION
ARG _JAVA_OPTIONS

WORKDIR /usr/src/app/

# Copy the native executable instead of JAR files
COPY --from=builder /usr/src/app/build/native/nativeCompile/ad-service ./ad-service
# Note: Native images don't use Java agents, so we skip the OpenTelemetry agent

EXPOSE ${AD_PORT}
ENTRYPOINT [ "./ad-service" ]
