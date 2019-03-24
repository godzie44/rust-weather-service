FROM rust:1.31.1-stretch as img-pool-build
USER root

RUN mkdir -p weather-service
COPY ./ ./weather-service

RUN cd /weather-service && cargo build --release

EXPOSE 8088

RUN mkdir -p /build-out \
&& cp /weather-service/target/release/weather_service /build-out/ \
&& cp -R /weather-service/cfg/ /cfg

USER root
CMD ["./build-out/weather_service"]

#билд: docker build -t weather-service .
#тесты: docker exec -t weather-service bash -c "cd \weather-service && cargo test"
#запуск docker run --rm --name weather-service --network="host" weather-service