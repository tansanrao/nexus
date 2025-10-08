# Troubleshooting Guide

Common issues and solutions for Linux KB with grokmirror.

## Grokmirror Issues

### Problem: "Mirror not found" error

**Symptoms:**
```
Mirror not found at ./api-server/mirrors/bpf/0.
Please ensure grokmirror is running and has completed at least one sync.
```

**Solutions:**

1. **Check if grokmirror is running:**
   ```bash
   ps aux | grep grok-pull
   ```

2. **Check if mirrors exist:**
   ```bash
   ls -la ./api-server/mirrors/
   ```

3. **Run grokmirror manually:**
   ```bash
   grok-pull -c grokmirror.conf
   ```
   First run takes 1-2 hours. Be patient!

4. **Check grokmirror logs:**
   ```bash
   tail -f ./api-server/mirrors/grokmirror.log
   ```

5. **Verify grokmirror config:**
   ```bash
   cat grokmirror.conf
   # Check that toplevel path is correct
   ```

### Problem: Grokmirror is slow or hangs

**Solutions:**

1. **Check network connection:**
   ```bash
   curl -I https://lore.kernel.org/manifest.js.gz
   ```

2. **Reduce pull_threads:**
   ```toml
   [pull]
   pull_threads = 2  # Down from 4
   ```

3. **Check disk space:**
   ```bash
   df -h
   # Need at least 20GB free
   ```

4. **Restart grokmirror:**
   ```bash
   killall grok-pull
   grok-pull -c grokmirror.conf
   ```

### Problem: Repository corruption

**Symptoms:**
```
Invalid git repository at ./api-server/mirrors/bpf/0.
The mirror may be corrupted.
```

**Solutions:**

1. **Run grok-fsck:**
   ```bash
   grok-fsck -c grokmirror.conf
   ```

2. **Delete corrupted repo and re-sync:**
   ```bash
   rm -rf ./api-server/mirrors/bpf
   grok-pull -c grokmirror.conf
   ```

3. **Check filesystem integrity:**
   ```bash
   # Linux
   sudo fsck /dev/sdX

   # macOS
   diskutil verifyVolume /
   ```

## Database Issues

### Problem: "No mailing lists available" in Sync panel

**Solution:**
1. Go to Settings → Database panel
2. Click "Seed Mailing Lists"
3. Wait for success message
4. Refresh Sync panel

### Problem: Seed button fails

**Symptoms:**
```
Failed to seed mailing lists: Connection refused
```

**Solutions:**

1. **Check API server is running:**
   ```bash
   curl http://localhost:8000/api/admin/database/status
   ```

2. **Check database connection:**
   ```bash
   psql -h localhost -U postgres -d linux-kernel-kb -c "SELECT 1"
   ```

3. **Reset database first:**
   ```bash
   # Via UI: Settings → Database → Reset Database
   ```

4. **Check API server logs:**
   ```bash
   # Terminal where you ran `cargo run`
   # Look for error messages
   ```

### Problem: Duplicate key errors

**Symptoms:**
```
duplicate key value violates unique constraint "mailing_lists_slug_key"
```

**Solution:**
The seed endpoint is idempotent, so this is fine. It means the list already exists.

If you want to re-seed from scratch:
1. Reset Database
2. Seed Mailing Lists

## Sync Issues

### Problem: Sync stuck at "Validating git mirrors"

**Solution:**
Grokmirror hasn't completed first sync yet. Wait for grokmirror to finish:

```bash
# Check progress
tail -f ./api-server/mirrors/grokmirror.log

# Check how many repos mirrored
ls -1 ./api-server/mirrors/ | wc -l
# Should be 341 when complete
```

### Problem: Sync processes 0 commits

**This is normal for incremental syncs!**

If there are no new emails since last sync:
- Commits discovered: 0
- Emails parsed: 0
- This is expected behavior

To verify incremental sync is working:
1. Enable a list
2. Run first sync (processes all commits)
3. Wait a few hours for new emails
4. Run second sync (processes only new commits)

### Problem: Sync takes too long

**Solutions:**

1. **Start with small lists:**
   - Enable `bpf` (single shard, moderate size)
   - Enable `sched-ext` (single shard, small)
   - Avoid `lkml` initially (18 shards, huge)

2. **Check database performance:**
   ```bash
   # PostgreSQL should have indexes
   psql -d linux-kernel-kb -c "\di"
   ```

3. **Increase PostgreSQL memory:**
   ```bash
   # Edit postgresql.conf
   shared_buffers = 256MB
   effective_cache_size = 1GB
   ```

4. **Run VACUUM:**
   ```bash
   psql -d linux-kernel-kb -c "VACUUM ANALYZE"
   ```

### Problem: Parse errors during sync

**Symptoms:**
```
Failed to parse email at commit abc123: Invalid header
```

**This is normal!**

Some emails in the archive have malformed headers. The sync will:
- Log warnings
- Continue processing
- Track parse_errors metric
- Import all valid emails

If parse errors are > 10%:
- Check grokmirror hasn't corrupted repos
- Run grok-fsck
- Report issue to lore.kernel.org if persistent

## Frontend Issues

### Problem: Frontend shows "Connection refused"

**Solutions:**

1. **Check API server is running:**
   ```bash
   curl http://localhost:8000/api/admin/database/status
   ```

2. **Check CORS settings:**
   API server should allow `http://localhost:5173`

3. **Check frontend is running:**
   ```bash
   cd frontend
   npm run dev
   # Should show: Local: http://localhost:5173
   ```

### Problem: Search doesn't work

**Solution:**
Search filters in memory, not the API. If you have 341 lists:
- Search should be instant
- If slow, check browser console for errors
- Try refreshing page

### Problem: Pagination stuck

**Solution:**
Click "Previous" to go back or reload page.

If consistently broken:
- Check browser console for errors
- Report bug with details

## Performance Issues

### Problem: API server high CPU

**Check what's running:**
```bash
# Get sync status
curl http://localhost:8000/api/admin/sync/status

# If sync is running, high CPU is normal
# If idle, check for infinite loops or leaked connections
```

### Problem: Database disk space growing

**This is expected!**

Email archives are large:
- lkml alone: ~1 million emails
- Full database: 10-50 GB depending on lists enabled

**Solutions:**

1. **Disable lists you don't need**
2. **Regular VACUUM:**
   ```bash
   psql -d linux-kernel-kb -c "VACUUM FULL"
   ```
3. **Archive old data** (future feature)

## Getting Help

### Check Logs

1. **API Server logs:**
   ```bash
   # Terminal where you ran `cargo run`
   # Set RUST_LOG for verbose output:
   RUST_LOG=debug cargo run --release
   ```

2. **Grokmirror logs:**
   ```bash
   tail -f ./api-server/mirrors/grokmirror.log
   ```

3. **Frontend logs:**
   ```bash
   # Browser console (F12)
   # Look for errors in red
   ```

4. **PostgreSQL logs:**
   ```bash
   # Linux
   tail -f /var/log/postgresql/postgresql-*.log

   # macOS (Postgres.app)
   ~/Library/Application\ Support/Postgres/var-*/postgresql.log
   ```

### Report Issues

When reporting issues, include:

1. **System info:**
   - OS and version
   - Rust version: `rustc --version`
   - Node version: `node --version`
   - PostgreSQL version: `psql --version`

2. **Error messages:**
   - Full error text
   - Stack traces if available

3. **Steps to reproduce:**
   - What you did
   - What you expected
   - What actually happened

4. **Logs:**
   - API server logs (last 50 lines)
   - Browser console errors
   - grokmirror logs if relevant

### Resources

- **Grokmirror docs:** https://github.com/mricon/grokmirror
- **Lore.kernel.org:** https://www.kernel.org/lore.html
- **PostgreSQL docs:** https://www.postgresql.org/docs/
- **Project README:** [README.md](./README.md)
- **Setup guide:** [GROKMIRROR_SETUP.md](./GROKMIRROR_SETUP.md)

## Common Warnings (Safe to Ignore)

### Rust warnings
```
warning: unused import: `Author`
warning: unused imports: `SortOrder`, ...
```
These are benign and don't affect functionality.

### PostgreSQL notices
```
NOTICE: relation "xyz" does not exist, skipping
```
Normal during database reset.

### Frontend build warnings
```
(!) Some chunks are larger than 500 KiB after minification
```
Expected for React apps with large dependencies.

## Quick Diagnosis

**Problem:** Nothing works
```bash
# Check everything is running:
ps aux | grep grok-pull       # Should show grokmirror
ps aux | grep cargo            # Should show api-server
lsof -i :8000                  # Should show api-server on port 8000
lsof -i :5173                  # Should show vite on port 5173
psql -d linux-kernel-kb -c "SELECT COUNT(*) FROM mailing_lists"  # Should show 341 or 0
```

**Problem:** Can't sync
```bash
# Check the chain:
1. ls ./api-server/mirrors/bpf/0  # Mirror exists?
2. psql -d linux-kernel-kb -c "SELECT * FROM mailing_lists WHERE slug='bpf'"  # List exists?
3. psql -d linux-kernel-kb -c "SELECT * FROM mailing_list_repositories WHERE mailing_list_id=(SELECT id FROM mailing_lists WHERE slug='bpf')"  # Repo configured?
4. curl -X POST http://localhost:8000/api/admin/sync/queue -H "Content-Type: application/json" -d '{"mailing_list_slugs":["bpf"]}'  # Can queue?
```

**Problem:** Not sure what's wrong
```bash
# Enable debug logging:
export RUST_LOG=debug
cargo run --release

# Then try to reproduce issue
# Logs will be VERY verbose
```
