use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
pub enum Command {
    #[command(description = "the default start command\n")]
    Start,
    #[command(description = "display this text\n")]
    Help,
    #[command(description = "roll a dice\n")]
    Dice,
    #[command(
        description = "get the cut out version of the gxp file \nArguments: filename, distance, address \nExample: `/cutgpx MyGpxFile.gpx 20 random_sreeet+random_city`\n",
        parse_with = "split"
    )]
    AddressCut {
        filename: String,
        distance: u32,
        start_address: String,
    },
    #[command(
        description = "get the cut out version of the gxp file from coordinates\nArguments: filename, distance, longitude latitude\n",
        parse_with = "split"
    )]
    CoordinatesCut {
        filename: String,
        distance: u32,
        longitude: String,
        latitude: String,
    },
    #[command(
        description = "delete the file with the given name\n Arguments: filename\nExample: `/deletefile MyGpxFile.gpx`\n"
    )]
    DeleteFile { filename: String },
    #[command(description = "list the stored files available\n")]
    ListFiles,
}
