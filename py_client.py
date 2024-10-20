import requests

def test_server(url='http://localhost:8081'):
    try:
        # GET request
        response = requests.get(url)
        print(f"GET Response (Status Code: {response.status_code}):")
        print(response.text)
        print("\nHeaders:")
        for key, value in response.headers.items():
            print(f"{key}: {value}")


    except requests.exceptions.RequestException as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    test_server()
