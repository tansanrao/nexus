# Grokmirror Setup for Linux KB

This guide explains how to set up grokmirror to mirror all lore.kernel.org mailing list archives for use with Linux KB.

## Overview

Linux KB uses a two-component architecture:

1. **Grokmirror** (external): Efficiently mirrors all ~341 lore.kernel.org git repositories
2. **API Server** (internal): Reads from local mirrors, parses emails, imports to database

This separation allows:
- Efficient rsync-like delta transfers via grokmirror
- Continuous background syncing independent of the API server
- Standard tooling recommended by kernel.org
- All lists mirrored; user chooses which to parse

## Prerequisites

- Python 3.6+ with pip
- ~20GB+ disk space (grows over time)
- Network connectivity to lore.kernel.org

## Installation

### 1. Install Grokmirror

```bash
pip install grokmirror
```

Verify installation:
```bash
grok-pull --version
```

### 2. Configure Grokmirror

The `grokmirror.conf` file in the project root is pre-configured for Linux KB.

Edit the `toplevel` path if needed:
```toml
[core]
toplevel = ./api-server/mirrors  # Adjust if needed
```

### 3. Initial Mirror Sync

Run the first sync manually to download all archives (~20GB+):

```bash
grok-pull -c grokmirror.conf
```

This will take several hours on the first run. Subsequent runs only pull changes and are much faster.

## Deployment Options

Choose one of the following deployment methods to keep mirrors up-to-date:

### Option 1: Systemd Service (Recommended)

Create a systemd user service to run grokmirror as a daemon.

**Create** `~/.config/systemd/user/grokmirror.service`:

```ini
[Unit]
Description=Grokmirror sync for lore.kernel.org
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/path/to/linux-kernel-kb
ExecStart=/usr/local/bin/grok-pull -c grokmirror.conf --daemon
Restart=always
RestartSec=60

[Install]
WantedBy=default.target
```

**Update the paths**:
- `WorkingDirectory`: Absolute path to your linux-kernel-kb directory
- `ExecStart`: Path to grok-pull (find with `which grok-pull`)

**Enable and start the service**:

```bash
systemctl --user daemon-reload
systemctl --user enable grokmirror.service
systemctl --user start grokmirror.service
```

**Check status**:

```bash
systemctl --user status grokmirror.service
journalctl --user -u grokmirror -f  # View logs
```

### Option 2: Cron Job

Add a cron job to run grokmirror every 5 minutes:

```bash
crontab -e
```

Add this line:
```cron
*/5 * * * * cd /path/to/linux-kernel-kb && grok-pull -c grokmirror.conf >> /tmp/grokmirror.log 2>&1
```

**Note**: Update `/path/to/linux-kernel-kb` with the actual path.

### Option 3: Manual Execution

For testing or development, run manually:

```bash
cd /path/to/linux-kernel-kb
grok-pull -c grokmirror.conf
```

## Repository Maintenance

Grokmirror includes `grok-fsck` for repository maintenance (repacking, corruption checks).

### Run Weekly Maintenance

Add to crontab (runs every Sunday at 2 AM):

```cron
0 2 * * 0 cd /path/to/linux-kernel-kb && grok-fsck -c grokmirror.conf
```

Or with systemd timer:

**Create** `~/.config/systemd/user/grokmirror-fsck.service`:

```ini
[Unit]
Description=Grokmirror repository maintenance

[Service]
Type=oneshot
WorkingDirectory=/path/to/linux-kernel-kb
ExecStart=/usr/local/bin/grok-fsck -c grokmirror.conf
```

**Create** `~/.config/systemd/user/grokmirror-fsck.timer`:

```ini
[Unit]
Description=Run grokmirror fsck weekly

[Timer]
OnCalendar=weekly
Persistent=true

[Install]
WantedBy=timers.target
```

**Enable the timer**:

```bash
systemctl --user daemon-reload
systemctl --user enable grokmirror-fsck.timer
systemctl --user start grokmirror-fsck.timer
```

## Monitoring

### Check Sync Status

View the log file:
```bash
tail -f ./api-server/mirrors/grokmirror.log
```

### List Mirrored Repositories

```bash
ls -la ./api-server/mirrors/
```

You should see directories for each mailing list (e.g., `lkml/`, `bpf/`, `netdev/`, etc.)

### Check Disk Usage

```bash
du -sh ./api-server/mirrors
```

## Using with Linux KB

Once grokmirror is running:

1. **Start the API server**:
   ```bash
   cd api-server && cargo run --release
   ```

2. **Reset and seed the database** (first time only):
   ```bash
   # Via curl
   curl -X POST http://localhost:8000/api/admin/database/reset
   curl -X POST http://localhost:8000/api/admin/mailing-lists/seed
   ```

3. **Enable mailing lists** in the web UI:
   - Navigate to http://localhost:5173/settings
   - Go to "Mailing Lists" tab
   - Toggle "Enabled" for lists you want to parse
   - Only enabled lists will be parsed; all lists remain mirrored

4. **Start syncing**:
   - Click "Sync" for the desired mailing list
   - The API server will parse emails from the local mirror
   - Subsequent syncs only process new commits (incremental)

## Troubleshooting

### Grokmirror Not Finding Repositories

**Problem**: `grok-pull` reports no repositories

**Solution**: Check that the manifest is accessible:
```bash
curl -s https://lore.kernel.org/manifest.js.gz | gunzip | head
```

### Disk Space Issues

**Problem**: Running out of disk space

**Solution**:
- Ensure you have 20GB+ free space
- Run `grok-fsck` to repack repositories
- Consider mirroring only specific lists (edit `grokmirror.conf` include patterns)

### API Server Can't Find Mirrors

**Problem**: API server reports "Mirror not found"

**Solution**:
- Verify `MIRROR_BASE_PATH` env var matches grokmirror's `toplevel`
- Check that repositories exist: `ls api-server/mirrors/bpf/0`
- Ensure grokmirror has completed at least one successful sync

### Permission Issues

**Problem**: Permission denied errors

**Solution**:
- Ensure the user running grokmirror has write access to `toplevel`
- Ensure the user running the API server has read access to mirrors

## Best Practices

1. **Run grokmirror continuously**: Use systemd or cron for automatic updates
2. **Don't sync too frequently**: Every 5 minutes is reasonable; faster may hit rate limits
3. **Monitor disk usage**: Repositories grow over time; plan for expansion
4. **Regular maintenance**: Run `grok-fsck` weekly to keep repos healthy
5. **Keep grokmirror updated**: `pip install --upgrade grokmirror`

## Additional Resources

- [Grokmirror GitHub](https://github.com/mricon/grokmirror)
- [Kernel.org Lore Documentation](https://www.kernel.org/lore.html)
- [Subscribing to Lore with Grokmirror](https://people.kernel.org/monsieuricon/subscribing-to-lore-lists-with-grokmirror)

## Support

For grokmirror issues, contact: tools@linux.kernel.org

For Linux KB issues, file an issue at: [project repository]
