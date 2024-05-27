import rsk
import zmq
from math import pi

# context = zmq.Context()

# # Creating subscriber connection
# sub = context.socket(zmq.SUB)
# sub.set_hwm(1)
# sub.connect("tcp://127.0.0.1:7557")
# sub.subscribe("")

# req = context.socket(zmq.REQ)
# req.connect("tcp://127.0.0.1:7558")

# while True:
#     print(sub.recv())
#     # json = sub.recv_json()
#     # print(json)

with rsk.Client(key="jFNNY") as client:
    def update(client, dt):
        client.blue1.goto((0., 0., pi), wait=False)
        client.blue2.goto((0., 0., pi), wait=False)
        client.green1.goto((0., 0., pi), wait=False)
        client.green2.goto((0., 0., pi), wait=False)
        print(client.ball, client.blue1.position)

    # client.on_update = update
    while True:
        print(client.ball, client.blue1.position, client.blue1.orientation)
        (x, y) = client.ball
        client.blue1.goto((x, y, pi/3.), wait=False)
        # client.blue2.goto((0., 0., pi), wait=False)
        client.blue2.goto((x, y, pi/3.), wait=False)
        # client.green2.goto((0., 0., pi), wait=False)