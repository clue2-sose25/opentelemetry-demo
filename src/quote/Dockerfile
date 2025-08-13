# Build stage
FROM maven:3.9.5-eclipse-temurin-21 AS build

WORKDIR /app
COPY /src/quote/pom.xml .
COPY /src/quote/src ./src

# Build the application
RUN mvn clean package -DskipTests

# Runtime stage
FROM eclipse-temurin:21-jre

WORKDIR /app

# Copy the built JAR from the build stage
COPY --from=build /app/target/quote-service-knative-1.0.0.jar app.jar

# Expose the port that the application runs on
EXPOSE 8080

# Set the default command to run the application
ENTRYPOINT ["java", "-jar", "app.jar"]
