import discord
from discord import ui
import logging
import re
import httpx


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

    async def on_submit(self, interaction: discord.Interaction):
        review = ""
        payload = dict()
        errors = False
        for (text_input, day) in zip(self.children, self.days):
            text = str(text_input)
            if not text:
                continue

            if not self.time_regex.fullmatch(text):
                review += f"{day}: FORMAT ERROR ({text})\n"
                logging.warn(f'TextInput {day} is of incorrect format: "{text}"')
                errors = True
            else:
                split = text.split(" - ")
                payload[day] = (split[0], split[1])

                review += f"{day}: from {split[0]} until {split[1]}\n"
                logging.info(f'TextInput {day}: "{text}"')

        if errors:
            await interaction.response.send_message(
                f"Some of the input fields were incorrectly formatted.\nPlease try again.\n{review}",
                ephemeral=True,
            )
        else:

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
