import asyncio
import os

import discord
from discord.ext import commands
from discord import Interaction
from typing import Literal, Optional
from discord.ext.commands import Greedy, Context

from timetable import Timetable

import logging

discord.utils.setup_logging()

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)


@bot.event
async def on_ready():
    logging.info(f"Logged in as {bot.user}")


@bot.tree.command()
async def timetable(interaction: Interaction):
    await interaction.response.send_modal(Timetable())


@bot.command()
@commands.guild_only()
@commands.is_owner()
async def sync(
    ctx: Context,
    guilds: Greedy[discord.Object],
    spec: Optional[Literal["~", "*", "^"]] = None,
) -> None:
    if not guilds:
        if spec == "~":
            synced = await ctx.bot.tree.sync(guild=ctx.guild)
        elif spec == "*":
            ctx.bot.tree.copy_global_to(guild=ctx.guild)
            synced = await ctx.bot.tree.sync(guild=ctx.guild)
        elif spec == "^":
            ctx.bot.tree.clear_commands(guild=ctx.guild)
            await ctx.bot.tree.sync(guild=ctx.guild)
            synced = []
        else:
            synced = await ctx.bot.tree.sync()
        msg = f"Synced {len(synced)} commands {'globally' if spec is None else 'to the current guild.'}"
        logging.info(msg)
        await ctx.send(msg)
        return

    ret = 0
    for guild in guilds:
        try:
            await ctx.bot.tree.sync(guild=guild)
        except discord.HTTPException:
            pass
        else:
            ret += 1
    msg = f"Synced the tree to {ret}/{len(guilds)}."
    logging.info(msg)
    await ctx.send(msg)


async def main():
    async with bot:
        # do you setup stuff if you need it here, then:
        bot.tree.copy_global_to(
            guild=discord.Object(id=1074973362268426290)
        )  # we copy the global commands we have to a guild, this is optional
        TOKEN = os.getenv("BOT_TOKEN")
        if TOKEN:
            await bot.start(TOKEN)


if __name__ == "__main__":
    asyncio.run(main())
