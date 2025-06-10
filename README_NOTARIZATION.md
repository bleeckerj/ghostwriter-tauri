* All the environment varialbes in .env.notarization
I created an App-Specific Passwords to verify my account during notarization
    https://account.apple.com/account/manage
I guess these are one-off passwords you can use

Thankfully I had downloaded the private key when I created the certificate.

I put that in 1Password. The file is named AuthKey_83XA5MYQ6J.p8 I saved it in 1Password

Then there was the certifcate business: convert the p8 to base64 text or something.

Then that actual base64 text is an environment variable as well - not the path to it, the actual data.

Then it almost got there, until I got an error about libpdfium.dylib not being signed.

So that's when I did my best to figure out how the heck you sign that kind of thing.

I thought maybe I'd have to build it, but I guess maybe not?

```
❯ codesign --force --verify --verbose --sign "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)" ./resources/libpdfium.dylib

./resources/libpdfium.dylib: replacing existing signature

./resources/libpdfium.dylib: signed Mach-O thin (arm64) [libpdfium]
```

Let's see if that takes hold.




* I had to codesign libpdfium.dylib

```
❯ security find-identity -v -p codesigning
  1) 22D237BF88691C37EAC3A9712E7BB8267125A514 "Apple Development: Julian Bleecker (93F8G7B58C)"
  2) AA9F584FC4EDC4BBA056CDDAB8A43B1A521FE983 "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
     2 valid identities found
```

* Turns out I was signing the libpdfium.dylib that was in the wrong place. I think it was just a copy I downloaded for safe keeping. The right one is in src-taur/resources, not ./resources

Then when I run ` npm run tauri:build:notarized ` it signs and bundles and everything.

I used 


```
    Finished `release` profile [optimized] target(s) in 6m 59s
       Built [tauri_cli::build] application at: /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/Ghostwriter
    Bundling [tauri_bundler::bundle::macos::app] Ghostwriter.app (/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app)
     Running [tauri_macos_sign] Command `base64  --decode -i /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpvxQNfH/src -o /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpaTndEU/cert.p12`
     Running [tauri_macos_sign] Command `security  create-keychain -p xLhYegWci56K4qi7 /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
     Running [tauri_macos_sign] Command `security  unlock-keychain -p xLhYegWci56K4qi7 /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
     Running [tauri_macos_sign] Command `security  import /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpaTndEU/cert.p12 -P fr00tl00ps -T /usr/bin/codesign -T /usr/bin/pkgbuild -T /usr/bin/productbuild -k /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
1 identity imported.
     Running [tauri_macos_sign] Command `security  set-keychain-settings -t 3600 -u /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
     Running [tauri_macos_sign] Command `security  set-key-partition-list -S apple-tool:,apple:,codesign: -s -k xLhYegWci56K4qi7 /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
keychain: "/Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db"
version: 512
class: 0x00000010 
attributes:
    0x00000000 <uint32>=0x00000010 
    0x00000001 <blob>="Mac Developer ID Application: Julian Bleecker"
    0x00000002 <blob>=<NULL>
    0x00000003 <uint32>=0x00000001 
    0x00000004 <uint32>=0x00000000 
    0x00000005 <uint32>=0x00000000 
    0x00000006 <blob>=0xCBA132699E6E44085678E73B8ED0AAFE0673ACCD  "\313\2412i\236nD\010Vx\347;\216\320\252\376\006s\254\315"
    0x00000007 <blob>=<NULL>
    0x00000008 <blob>=0x7B38373139316361322D306663392D313164342D383439612D3030303530326235323132327D00  "{87191ca2-0fc9-11d4-849a-000502b52122}\000"
    0x00000009 <uint32>=0x0000002A  "\000\000\000*"
    0x0000000A <uint32>=0x00000800 
    0x0000000B <uint32>=0x00000800 
    0x0000000C <blob>=0x0000000000000000 
    0x0000000D <blob>=0x0000000000000000 
    0x0000000E <uint32>=0x00000001 
    0x0000000F <uint32>=0x00000001 
    0x00000010 <uint32>=0x00000001 
    0x00000011 <uint32>=0x00000000 
    0x00000012 <uint32>=0x00000001 
    0x00000013 <uint32>=0x00000001 
    0x00000014 <uint32>=0x00000001 
    0x00000015 <uint32>=0x00000001 
    0x00000016 <uint32>=0x00000001 
    0x00000017 <uint32>=0x00000001 
    0x00000018 <uint32>=0x00000001 
    0x00000019 <uint32>=0x00000001 
    0x0000001A <uint32>=0x00000001 
     Running [tauri_macos_sign] Command `security  list-keychain -d user -s /Users/julian/Library/Keychains/login.keychain-db /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
found cert "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)" with organization "Julian Bleecker"
     Running [tauri_bundler::utils] Command `xattr  -crs /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app`
     Signing [tauri_bundler::bundle::macos::sign] with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app/Contents/MacOS/Ghostwriter
     Running [tauri_macos_sign] Command `codesign  --force -s Developer ID Application: Julian Bleecker (9DCTLJ9BY7) --options runtime --keychain /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app/Contents/MacOS/Ghostwriter`
/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app/Contents/MacOS/Ghostwriter: replacing existing signature
Signing with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app
     Running [tauri_macos_sign] Command `codesign  --force -s Developer ID Application: Julian Bleecker (9DCTLJ9BY7) --options runtime --keychain /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app`
/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app: replacing existing signature
     Running [tauri_macos_sign] Command `ditto  -c -k --keepParent --sequesterRsrc /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpcUhcTS/Ghostwriter.zip`
Signing with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpcUhcTS/Ghostwriter.zip
     Running [tauri_macos_sign] Command `codesign  --force -s Developer ID Application: Julian Bleecker (9DCTLJ9BY7) --keychain /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpcUhcTS/Ghostwriter.zip`
Notarizing /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app
Notarizing Finished with status Accepted for id 7e2b6914-c3e7-475e-accc-de6635a032b2 (Processing complete)
     Running [tauri_macos_sign] Command `security  delete-keychain /Users/julian/Library/Keychains/9217Hac2B0tnaVie.keychain-db`
    Bundling [tauri_bundler::bundle::macos::dmg] Ghostwriter_0.2.9_aarch64.dmg (/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/Ghostwriter_0.2.9_aarch64.dmg)
     Running [tauri_bundler::bundle::macos::dmg] bundle_dmg.sh
     Running [tauri_bundler::utils] Command `/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/bundle_dmg.sh  --volname Ghostwriter --icon Ghostwriter.app 180 170 --app-drop-link 480 170 --window-size 660 400 --hide-extension Ghostwriter.app --volicon /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/icon.icns Ghostwriter_0.2.9_aarch64.dmg Ghostwriter.app`
Creating disk image...
created: /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/rw.36187.Ghostwriter_0.2.9_aarch64.dmg
Mounting disk image...
Device name:     /dev/disk15
Searching for mounted interstitial disk image using /dev/disk15s...
Mount dir:       /Volumes/dmg.dOLzqd
Making link to Applications dir...
/Volumes/dmg.dOLzqd
Copying volume icon file '/Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/icon.icns'...
Will sleep for 2 seconds to workaround occasions "Can't get disk (-1728)" issues...
Running AppleScript to make Finder stuff pretty: /usr/bin/osascript "/var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/createdmg.tmp.XXXXXXXXXX.95Shnb5BjZ" "dmg.dOLzqd"
waited 1 seconds for .DS_STORE to be created.
Done running the AppleScript...
Fixing permissions...
Done fixing permissions
Skipping blessing on sandbox
Deleting .fseventsd
Unmounting disk image...
"disk15" ejected.
Compressing disk image...
Preparing imaging engine…
Reading Protective Master Boot Record (MBR : 0)…
   (CRC32 $18C47172: Protective Master Boot Record (MBR : 0))
Reading GPT Header (Primary GPT Header : 1)…
   (CRC32 $0875F88F: GPT Header (Primary GPT Header : 1))
Reading GPT Partition Data (Primary GPT Table : 2)…
   (CRC32 $9CE365BB: GPT Partition Data (Primary GPT Table : 2))
Reading  (Apple_Free : 3)…
   (CRC32 $00000000:  (Apple_Free : 3))
Reading disk image (Apple_HFS : 4)…
   (CRC32 $1A7366EA: disk image (Apple_HFS : 4))
Reading  (Apple_Free : 5)…
   (CRC32 $00000000:  (Apple_Free : 5))
Reading GPT Partition Data (Backup GPT Table : 6)…
   (CRC32 $9CE365BB: GPT Partition Data (Backup GPT Table : 6))
Reading GPT Header (Backup GPT Header : 7)…
   (CRC32 $C9652841: GPT Header (Backup GPT Header : 7))
Adding resources…
Elapsed Time:  2.670s
File size: 19813302 bytes, Checksum: CRC32 $B039A55E
Sectors processed: 129103, 79284 compressed
Speed: 14.5MB/s
Savings: 70.0%
created: /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter_0.2.9_aarch64.dmg
hdiutil does not support internet-enable. Note it was removed in macOS 10.15.
Disk image done
     Running [tauri_macos_sign] Command `base64  --decode -i /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpd7Nuxg/src -o /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpgynaY5/cert.p12`
     Running [tauri_macos_sign] Command `security  create-keychain -p 3525VrrRlc5Gh5yK /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
     Running [tauri_macos_sign] Command `security  unlock-keychain -p 3525VrrRlc5Gh5yK /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
     Running [tauri_macos_sign] Command `security  import /var/folders/0r/vdxl1k_16mq2083pkr4qqp040000gn/T/.tmpgynaY5/cert.p12 -P fr00tl00ps -T /usr/bin/codesign -T /usr/bin/pkgbuild -T /usr/bin/productbuild -k /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
1 identity imported.
     Running [tauri_macos_sign] Command `security  set-keychain-settings -t 3600 -u /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
     Running [tauri_macos_sign] Command `security  set-key-partition-list -S apple-tool:,apple:,codesign: -s -k 3525VrrRlc5Gh5yK /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
keychain: "/Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db"
version: 512
class: 0x00000010 
attributes:
    0x00000000 <uint32>=0x00000010 
    0x00000001 <blob>="Mac Developer ID Application: Julian Bleecker"
    0x00000002 <blob>=<NULL>
    0x00000003 <uint32>=0x00000001 
    0x00000004 <uint32>=0x00000000 
    0x00000005 <uint32>=0x00000000 
    0x00000006 <blob>=0xCBA132699E6E44085678E73B8ED0AAFE0673ACCD  "\313\2412i\236nD\010Vx\347;\216\320\252\376\006s\254\315"
    0x00000007 <blob>=<NULL>
    0x00000008 <blob>=0x7B38373139316361322D306663392D313164342D383439612D3030303530326235323132327D00  "{87191ca2-0fc9-11d4-849a-000502b52122}\000"
    0x00000009 <uint32>=0x0000002A  "\000\000\000*"
    0x0000000A <uint32>=0x00000800 
    0x0000000B <uint32>=0x00000800 
    0x0000000C <blob>=0x0000000000000000 
    0x0000000D <blob>=0x0000000000000000 
    0x0000000E <uint32>=0x00000001 
    0x0000000F <uint32>=0x00000001 
    0x00000010 <uint32>=0x00000001 
    0x00000011 <uint32>=0x00000000 
    0x00000012 <uint32>=0x00000001 
    0x00000013 <uint32>=0x00000001 
    0x00000014 <uint32>=0x00000001 
    0x00000015 <uint32>=0x00000001 
    0x00000016 <uint32>=0x00000001 
    0x00000017 <uint32>=0x00000001 
    0x00000018 <uint32>=0x00000001 
    0x00000019 <uint32>=0x00000001 
    0x0000001A <uint32>=0x00000001 
     Running [tauri_macos_sign] Command `security  list-keychain -d user -s /Users/julian/Library/Keychains/login.keychain-db /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
found cert "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)" with organization "Julian Bleecker"
     Signing [tauri_bundler::bundle::macos::sign] with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing with identity "Developer ID Application: Julian Bleecker (9DCTLJ9BY7)"
Signing /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/Ghostwriter_0.2.9_aarch64.dmg
     Running [tauri_macos_sign] Command `codesign  --force -s Developer ID Application: Julian Bleecker (9DCTLJ9BY7) --keychain /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/Ghostwriter_0.2.9_aarch64.dmg`
     Running [tauri_macos_sign] Command `security  delete-keychain /Users/julian/Library/Keychains/M5A3QwB5gbqFoiNN.keychain-db`
    Finished [tauri_bundler::bundle] 2 bundles at:
        /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/macos/Ghostwriter.app
        /Users/julian/Code/tiptap-stuff/ghostwriter-tauri/src-tauri/target/release/bundle/dmg/Ghostwriter_0.2.9_aarch64.dmg
```