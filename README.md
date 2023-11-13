# rs-screenshot-uploader

rs-screenshot-uploader is a tool that allows you to automatically upload screenshots from specified folders to your Telegram chat. This repository contains the source code for the rs-screenshot-uploader application.

## Configuration

Before using the rs-screenshot-uploader, you need to configure it by specifying the folders to be watched and providing your Telegram bot token and chat ID. Follow the instructions below:

1. Rename the `config.example.toml` file in the root directory of this repository to `config.toml`.

2. Open the `config.toml` file in a text editor.

3. Specify the paths of the folders you want to watch by modifying the `path` field. Add the full paths of the folders as elements of the array. For example:

```toml
path = [
  "C:/Users/User/Pictures/VRChat",
  "C:/Users/jsopn/Pictures/Holoswitch"
]
```

4. Obtain a bot token for your Telegram bot from [@BotFather](https://t.me/BotFather). Replace `long_bot_token_string_from_botfather_in_telegram` in the `token` field with your generated bot token.

```toml
token = "your_bot_token_string_from_botfather_in_telegram"
```

5. Give the bot you created admin rights in the chat or channel.

6. Obtain your Telegram chat ID using [@getmy_idbot](https://t.me/getmy_idbot). Replace the empty `chat_id` field with your Telegram chat ID.

```toml
chat_id = "your_telegram_chat_id"
```

7. Save the `config.toml` file.

## Usage

After configuring the rs-screenshot-uploader, you can run the application and it will automatically monitor the specified folders for new screenshots. Once a screenshot is detected, it will be uploaded to your configured Telegram chat.

## License

This repository is licensed under the [Do What the Fuck You Want To Public License](LICENSE).
