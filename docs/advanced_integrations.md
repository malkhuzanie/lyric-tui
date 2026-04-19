# Advanced Media Player Integration

Whilst `lyric-tui` seamlessly detects most mainstream media players out of the box, certain specialised audio players require minor manual intervention to expose their playback data to the host operating system.

## Roon Integration (Linux Specific)

_Please note: This guide pertains strictly to Linux distributions. As Windows and macOS utilise entirely distinct media control architectures (GSMTC and MediaRemote, respectively), this specific bridge daemon is not applicable to those platforms._

The Roon audio player does not natively broadcast its playback state via the MPRIS D-Bus interface. Consequently, for `lyric-tui` to detect and synchronise with Roon's audio stream, you must utilise a third-party bridge daemon.

We highly recommend the `roon-extension-mpris` project to accomplish this. To install and initialise this service, please ensure you have Node.js installed on your system, and subsequently execute the following commands within your terminal:

```
# 1. Clone the extension repository
git clone [https://github.com/NoaHimesaka1873/roon-extension-mpris.git](https://github.com/NoaHimesaka1873/roon-extension-mpris.git)

# 2. Navigate into the newly created directory
cd roon-extension-mpris

# 3. Install the requisite Node dependencies
npm install

# 4. Initialise the background daemon
npm start
```

**Crucial Next Step:** Upon successfully initialising the daemon, you must open your Roon control application, navigate to **Settings > Extensions**, and explicitly enable the newly discovered `roon-extension-mpris` module.

_Note: Once the daemon is actively running and this authorisation is granted within Roon, `lyric-tui` shall automatically detect Roon as an active MPRIS player without any further configuration required on your part._