# Weather-service, сервис для получения прогноза погоды из нескольких источников.

# Запуск (docker):

Из директории с проектом:
```` 
docker build -t weather-service .
docker run --rm --name weather-service --network="host" weather-service
````

# End points

#### GET http://localhost:8088/weather/{location}/on/{date}
Прогноз погоды на конкертный день.
<br> location - город
<br> date - дата в формате Ymd

Пример: http://localhost:8088/weather/Moscow/on/2019-03-26
Ответ:
````
{
  "ok": true,
  "forecast": {
    "2019-03-26": 1.75
  },
  "warnings": []
}
````

##### Возможные ошибки:
Если один из истоников не может отдать прогноз на заданную дата дату (или для заданной location) запись об этом будет в warnings
````
{
  "ok": true,
  "forecast": {
    "2019-03-31": 4.0
  },
  "warnings": [
    "Unsupported date 2019-03-31 for provider Apixu !"
  ]
}
````

Если данные нельзя получить ни из одного источника - в ответе будет ok = false, в warnings - полный список ошибок.
````
{
  "ok": false,
  "forecast": null,
  "warnings": [
    "Unsupported date 2019-06-10 for provider Apixu !",
    "Unsupported date 2019-06-10 for provider Yahoo !"
  ]
}
````

#### GET http://localhost:8088/weather/{location}/week
Прогноз погоды на текущий день + 4 дня.
<br>location - город

Пример: http://localhost:8088/weather/Belgorod/week
Ответ:
````
{
  "ok": true,
  "forecast": {
    "2019-03-24": 1.95,
    "2019-03-25": 3.75,
    "2019-03-26": 4.85,
    "2019-03-27": 2.25,
    "2019-03-28": 1.0
  },
  "warnings": []
}
````
##### Возможные ошибки:
См. предыдущий end point

# Тесты

```` 
docker exec -t weather-service bash -c "cd \weather-service && cargo test"
```` 

