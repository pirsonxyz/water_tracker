import requests
from requests.sessions import Request;
water_intake = int(input("Enter the water intake: "))
target = int(input("Enter target: "))
r = requests.post("http://localhost:3000/add_water", json={
    "water_intake": water_intake,
    "target": target,
})
#r = requests.get("http://localhost:3000")
print(r.json())
new = int(input("Enter new intake: "))
u = requests.post("http://localhost:3000/update_water", json={
    "water_intake": new
})
print(u.json())