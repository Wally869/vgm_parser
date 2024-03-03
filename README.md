
Header is 256 bytes

version 1.7 added an extra header 
used for additional chip clocks



https://vgmrips.net/wiki/VGM_Specification


check out metadata is Gd3 format:
https://www.smspower.org/uploads/Music/gd3spec100.txt  

so can parse from Gd3 string (so 47 64 33 20 bytes) 
 


/* "Track name (in English characters)\0"
"Track name (in Japanese characters)\0"
"Game name (in English characters)\0"
"Game name (in Japanese characters)\0"
"System name (in English characters)\0"
"System name (in Japanese characters)\0"
"Name of Original Track Author (in English characters)\0"
"Name of Original Track Author (in Japanese characters)\0"
"Date of game's release written in the form yyyy/mm/dd, or just yyyy/mm or yyyy if month and day is not known\0"
"Name of person who converted it to a VGM file.\0"
"Notes\0" */


All header sizes are valid for all versions from 1.50 on, as long as header has at least 64 bytes. If the VGM data starts at an offset that is lower than 0x100, all overlapping header bytes have to be handled as they were zero.



can reduce size of VGM with 
https://vgmrips.net/wiki/Vgm_cmp
guess it's done usually, but can still try tbh