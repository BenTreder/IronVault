# IronVault GUI User Flow

## First-time demo

1. Open IronVault.
2. Go to Settings.
3. Click Set up test vault.
4. Go to Backup.
5. Click Check setup.
6. Type SEAL.
7. Click Run backup.
8. Go to Snapshots.
9. Confirm snapshot details appear.
10. Go to Restore.
11. Preview restore.
12. Type RESTORE.
13. Restore into a clean folder.

## Real backup setup

1. Go to Settings.
2. Find Set up your own backup.
3. Choose the folder to back up.
4. Choose the backup storage folder.
5. Choose the IronVault settings file.
6. Click Create backup setup.
7. Go to Backup.
8. Check setup.
9. Type SEAL.
10. Run backup.

## Restore behavior

IronVault restores the original backed-up folder inside the restore folder.

Example original file:

/home/ben/BackupTest/TestFile.txt

Example restore folder:

/tmp/restore

Example restored file:

/tmp/restore/BackupTest/TestFile.txt

This keeps restored files grouped together and avoids dumping files loose into a folder.
