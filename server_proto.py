"""Protobuf API server — returns User and UserList as binary protobuf."""
import random
import string
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "generated"))

import uvicorn
from fastapi import FastAPI
from fastapi.responses import Response
import user_pb2

app = FastAPI()


def random_str(n=8):
    return "".join(random.choices(string.ascii_lowercase, k=n))


def make_user(uid: int) -> user_pb2.User:
    user = user_pb2.User(
        id=uid,
        name=f"{random_str(5)} {random_str(7)}",
        email=f"{random_str(6)}@{random_str(5)}.com",
        age=random.randint(18, 80),
        address=user_pb2.Address(
            street=f"{random.randint(1, 999)} {random_str(6)} St",
            city=random_str(7).capitalize(),
            country="US",
            zip_code=f"{random.randint(10000, 99999)}",
        ),
    )
    for _ in range(random.randint(1, 5)):
        user.orders.append(
            user_pb2.Order(
                order_id=random_str(12),
                product=random_str(8),
                price=round(random.uniform(1.0, 500.0), 2),
                quantity=random.randint(1, 20),
            )
        )
    return user


@app.get("/user/{uid}")
def get_user(uid: int):
    user = make_user(uid)
    return Response(
        content=user.SerializeToString(),
        media_type="application/x-protobuf",
    )


@app.get("/users")
def get_users(count: int = 100):
    user_list = user_pb2.UserList(users=[make_user(i) for i in range(count)])
    return Response(
        content=user_list.SerializeToString(),
        media_type="application/x-protobuf",
    )


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8002)
