import rsk
from math import pi

with rsk.Client() as client:
    def update(client, dt):
        client.blue1.goto((0., 0., pi), wait=False)
        client.blue2.goto((0., 0., pi), wait=False)
        client.green1.goto((0., 0., pi), wait=False)
        client.green2.goto((0., 0., pi), wait=False)
        print(client.ball, client.blue1.position)

    # client.on_update = update
    while True:
        (x, y) = client.ball
        client.blue1.goto((x, y, pi/3.), wait=False)
        # client.blue2.goto((0., 0., pi), wait=False)
        client.green1.goto((x, y, pi/3.), wait=False)
        # client.green2.goto((0., 0., pi), wait=False)
        print(client.ball, client.blue1.position, client.blue1.orientation)