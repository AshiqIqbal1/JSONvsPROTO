"""JSON API server — returns User and UserList as JSON."""
import random
import string
import uvicorn
from fastapi import FastAPI
from fastapi.responses import JSONResponse

app = FastAPI()


def random_str(n=8):
    return "".join(random.choices(string.ascii_lowercase, k=n))


def make_user(uid: int) -> dict:
    return {
        "id": uid,
        "name": f"{random_str(5)} {random_str(7)}",
        "email": f"{random_str(6)}@{random_str(5)}.com",
        "age": random.randint(18, 80),
        "address": {
            "street": f"{random.randint(1, 999)} {random_str(6)} St",
            "city": random_str(7).capitalize(),
            "country": "US",
            "zip_code": f"{random.randint(10000, 99999)}",
        },
        "orders": [
            {
                "order_id": random_str(12),
                "product": random_str(8),
                "price": round(random.uniform(1.0, 500.0), 2),
                "quantity": random.randint(1, 20),
            }
            for _ in range(random.randint(1, 5))
        ],
    }


@app.get("/user/{uid}")
def get_user(uid: int):
    return JSONResponse(content=make_user(uid))


@app.get("/users")
def get_users(count: int = 100):
    users = [make_user(i) for i in range(count)]
    return JSONResponse(content={"users": users})


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8001)
