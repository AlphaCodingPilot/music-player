A command line based music player that plays downloaded songs inside a folder

## Installation
Clone the repository:
```
git clone https://github.com/AlphaCodingPilot/music-player.git
```

## Usage
1. Create a folder named 'playlist' inside the 'music-player' directory (this repo)
2. Download songs as mp3 files and put them into the 'playlist' folder
3. Run the program by navigating to the 'music-player' directory and running the ```cargo run --release``` command (you need cargo and the rust compiler installed for this)
4. You can press F4 or F7 or type ```pause``` in the command line to pause/resume the audio player
5. Type ```commands``` in the command line to see all available commands

## Features
1. The playlist is shuffled by default and songs are chosen with a probability distribution that favors songs which have not been chosen repidetly. You can also manually increase the probability of a song to get chosen by 2x by "starring" it (typing ```star``` into the command line when the song is being played).
2. The audio player can be controlled with keyboard shortcuts (like F7 for pause/resume) even if the window is not in focus.
3. The volume of specific songs relative to all other songs can be manually adjusted (the ```song volume +``` and ```song volume -``` commands increase/decrease the song volume by 10%).
4. You can enter or exit focus mode by pressing F9 which prevents any songs with lyrics from being played. For this to work you need to mark songs as having lyrics by typing ```has lyrics``` into the command line, which will mark the currenty playing song as having lyrics. I recommend disabling shuffling by using the ```disable shuffle``` command to go through the playlist and mark any snogs with lyrics as such. You can type ```next``` or press F8 to skip to next song. When shuffling is disabled this will play the next song in the ```playlist``` folder.
5. Specified data like which songs have lyrics, which songs are starred and the individual song volumes of songs are persistent even when the program is closed and reopened.
