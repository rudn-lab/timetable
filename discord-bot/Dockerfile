FROM python:3

WORKDIR /app
CMD [ "python", "./bot.py" ]

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY . .
