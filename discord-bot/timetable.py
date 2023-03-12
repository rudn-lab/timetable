import discord
from discord import ui
import logging
import re
import httpx
from datetime import time


class Timetable(ui.Modal, title="New opening and closing times of the RUDN Lab"):
    days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
    time_format = "HH:MM - HH:MM"
    time_regex = re.compile(
        r"^(0[0-9]|1[0-9]|2[0-3]):[0-5][0-9] - (0[0-9]|1[0-9]|2[0-3]):[0-5][0-9]$"
    )

    def __init__(self) -> None:
        super().__init__()
        for day in self.days:
            day_input = ui.TextInput(
                label=day,
                style=discord.TextStyle.short,
                placeholder=self.time_format,
                min_length=len(self.time_format),
                max_length=len(self.time_format),
                required=False,
            )
            self.add_item(day_input)
        logging.info("New timetable request")

    def is_timeperiod_valid(self, from_time: time, until_time: time) -> bool:
        hour_delta = until_time.hour - from_time.hour
        if hour_delta > 0:
            return True
        elif hour_delta == 0:
            minute_delta = until_time.minute - from_time.minute
            if minute_delta > 0:
                return True

        return False

    async def on_submit(self, interaction: discord.Interaction):
        review = "```\n"
        payload = dict()
        errors = False
        for (text_input, day) in zip(self.children, self.days):
            text = str(text_input)
            if not text:
                continue

            if self.time_regex.fullmatch(text):
                split = text.split(" - ")
                from_time = time.fromisoformat(split[0])
                until_time = time.fromisoformat(split[1])

                if self.is_timeperiod_valid(from_time, until_time):
                    # Note: sending with seconds, so that CF worker easily parses update request
                    payload[day] = (
                        from_time.isoformat(),
                        until_time.isoformat(),
                    )

                    msg = f"{day}: {text}"
                    review += msg + "\n"
                    logging.info(msg)
                else:
                    msg = f"{day}: INVALID TIME PERIOD ({text})"
                    review += msg + "\n"
                    logging.warn(msg)
                    errors = True

            else:
                msg = f"{day}: POOR FORMATTING ({text})"
                review += msg + "\n"
                logging.warn(msg)
                errors = True

        review += "```"

        if errors:
            await interaction.response.send_message(
                f"Some of the input fields contain errors.\nPlease try again.\n{review}",
                ephemeral=True,
            )
        elif payload:

            button = ui.Button(label="Confirm")
            view = ui.View()
            view.add_item(button)

            async def post_new_timetable_info(interaction: discord.Interaction):
                headers = {"Content-Type": "application/json"}
                url = "https://timetable.rudn-lab.ru/update"
                async with httpx.AsyncClient() as client:
                    r = await client.post(url, headers=headers, json=payload)
                    if r.status_code == 200:
                        logging.info(f"Succesfully posted {payload} to {url}")
                    else:
                        logging.warn(f"Could not post {payload} to {url}")

                button.disabled = True
                button.label = "Confirmed"
                await interaction.response.edit_message(
                    content=f"This is the new timetable.\n{review}", view=view
                )

            button.callback = post_new_timetable_info

            await interaction.response.send_message(
                f"This will be the new timetable.\n{review}", view=view
            )
        else:
            await interaction.response.send_message(
                "No input. Nothing to update.", ephemeral=True
            )
