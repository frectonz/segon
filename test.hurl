POST http://127.0.0.1:3030/register
{
  "username": "test",
  "password": "test"
}

HTTP/1.1 201
[Asserts]
jsonpath "$.status" == "OK"


POST http://127.0.0.1:3030/login
{
  "username": "test",
  "password": "test"
}

HTTP/1.1 200
[Asserts]
jsonpath "$.status" == "OK"
jsonpath "$.data.username" == "test"
jsonpath "$.data.password" == "test"
