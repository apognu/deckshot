# Deckshot

**UNDER DEVELOPEMENT**

Deckshot is a service to be run on your Steam Deck that will automatically upload any screenshot taken while it is running. So far, it supports the following remote services:

 * S3
 * Google Drive
 * Dropbox
 * Microsoft OneDrive
 * imgur
 * Discord

It will try and find the name of the game played while the screenshot was taken and place it in an appropriately named folder. If the name of the game cannot be determined (network issue, non-Steam game, GameScope), the screenshot will be uploaded to an `UNKNOWN GAME` folder.

It maintains an internal (and simple) database storing the screenshots that failed to upload (for example, if you were offline), and will regularly retry those uploads.

## Installation

This tool is now installed through Decky and configured through a terminal (SSH or Desktop mode). Running it for the first time will create `/home/deck/.config/deckshot` and copy the template configuration file there.

## Configuration

Deckshot is configured by editing `/home/deck/.config/deckshot/deckshot.yml`. The only required configuration settings are the one defining the uploader to use. When you edit the configuration, restart deckshot by using the Decky plugin UI.

The upload providers requiring an interactive authentication process (open a browser for instance) can be configured by running the `auth` command once the configuration file is updated accordingly.

```shell
deck$ /home/deck/homebrew/plugins/deckshot/bin/deckshot auth
```

Here are the required parameters per uploader:

### S3 / Minio

As of this writing, only path-based S3 access is supported. Credentials with `s3:PutObject` allowed are required.

```yaml
uploader:
  kind: S3
  endpoint: <URL to your S3 endpoint>
  region: <region name>
  access_key_id: <AWS access key ID>
  secret_access_key: <AWS secret access key>
  bucket: <bucket name>
```

### Google Drive

The Google Drive integration requires a service account created from the [Google Developers console](https://console.cloud.google.com/iam-admin/serviceaccounts/create), in the form of a JSON private key. The service account itself does not require permission, but will need to autorize it to access a folder in your Google drive.

To do so, note the service account email address (`xxx@yyy@iam.gserviceaccount.com`), and share the folder with that email address, with write permissions. Also note the folder ID from the URL, you will need to indicate it in the configuration.

Finally, you will need to enable the [Google Drive API](https://console.cloud.google.com/marketplace/product/google/drive.googleapis.com).

```yaml
uploader:
  kind: GoogleDrive
  private_key_file: <path to service account JSON key>
  folder: <folder ID to use>
```

### Dropbox

The Dropbox integration requires that you create a Dropbox OAuth2 application from [Dropbox's Developers console](https://www.dropbox.com/developers/apps/create), once it is created, note down the `App key` shown in the "OAuth 2" section, and give it the `files.content.write` scope from the Permissions tab.

Depending on the appliction's access type you chose when creating it, the value of the `folder` parameters will either be appended to the application's folder, or will start at your Dropbox's root.

```yaml
uploader:
  kind: Dropbox
  client_id: <application's app key>
  folder: <path to the remote folder, either scoped to the app's folder, or from the root>
```

Now, run the authentication process with `deckshot auth` and follow the instructions before restarting deckshot.

### OneDrive

The OneDrive integration requires the creation of an OAuth2 application on the [Azure portal](https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade). From there, go to "New registration", and create a "Personal Microsoft accounts only" account. You can choose whichever `Web` Redirect URI you want, like `http://localhost:8080/redirect` (the loading of this page will fail anyway, we will retrieve the code from the URL). When the application is created, generate a client secret by going to "Certificates and secrets" and clicking "New client secret". Finally, give the application the `Files.ReadWrite` permission by going into "API permissions" (this is a "Microsoft Graph" delegated permission).

Report the values for the "Application (client) ID" (in the "Overview" tab), the client secret and the redirect URI in the configuration file.

```yaml
uploader:
  kind: OneDrive
  client_id: <Microsoft client ID>
  client_secret: <Microsoft client secret>
  redirect_uri: <redirect URI you entered>
  folder: <name of the folder to create and use>
```

# imgur

To upload your screenshots to imgur, register an OAuth 2 application [here](https://api.imgur.com/oauth2/addclient), add a dummy redirect URI, such as `http://127.0.0.1:8080/redirect`, and configure Deckshot with the provided information, run `deckshot auth` to start the authentication flow, and fill in the code from the redirect page's address bar.

```yaml
uploader:
  kind: Imgur
  client_id: <imgur client ID>
  client_secret: <imgur client secret>
  redirect_uri: <provided redirect URI>
```

# Discord

To post every screenshot to a Discord channel, start by creating a [Discord Developer Portal](https://discord.com/developers/applications), note the "Application ID", and create bot from it. From there, generate and copy the token, and visit the authorization at [https://discordapp.com/api/oauth2/authorize?client_id=APP_ID&permissions=2048&scope=bot](https://discordapp.com/api/oauth2/authorize?client_id=APP_ID&permissions=2048&scope=bot) (replace with your application ID), select the server you wish to post your screenshot in, and you can configure Deckshot.

```yaml
uploader:
  kind: Discord
  token: <bot token>
  channel: <channel ID>
  username: <your username if you wish your screenshots to be annotated with a username>
```
