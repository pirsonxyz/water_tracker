import requests

for i in range(1,101):
    try:
        url = f"http://10.0.0.{i}:3000/sanity"
        print(url)
        response = requests.get(url)
        print(response.status_code)
        if response.status_code == 200:
            print(f"Encontre url {url}")
            break;
    except:
        print("URL no es")
        continue;
