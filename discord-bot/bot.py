import discord
import os
import requests

import logging
discord.utils.setup_logging()

intents = discord.Intents.default()
intents.message_content = True

client = discord.Client(intents=intents)

@client.event
async def on_ready():
    logging.info(f"We have logged in as {client.user}")

@client.event
async def on_message(message):
    if message.author == client.user:
        return

    logging.info(f"Message from {message.author}: {message.content}")
    if message.content.startswith("$hello"):
        await message.channel.send("Hello!")
    elif message.content.startswith("$echo"):
        rest = message.content.strip("$echo ")
        await message.channel.send(rest)
    elif message.content.startswith("$post"):
        rest = message.content.stip("$post ")
        payload = {"msg": rest}
        headers = {
            "Content-Type": "application/json"
        }
        requests.post("https://timetable.rudn-lab.ru/update", json=payload, headers=headers)


TOKEN = os.getenv("BOT_TOKEN")
if TOKEN:
    client.run(TOKEN)
