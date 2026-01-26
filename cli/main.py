"""
Main CLI entry point.
Uses Click if available, otherwise falls back to argparse.
"""
import sys

# Try Click, fallback to argparse
try:
    import click
    HAS_CLICK = True
except ImportError:
    HAS_CLICK = False

from .output import out, HAS_RICH
from curseforge import CurseForgeClient, Config


def get_client_and_config():
    """Initialize config and client."""
    config = Config()

    if not config.api_key:
        out.error("API key not set. Run: hytale-cf config --api-key YOUR_KEY")
        sys.exit(1)

    client = CurseForgeClient(config.api_key)

    with out.status("Connecting to CurseForge"):
        if not client.init_connection():
            out.error("Failed to connect to CurseForge API")
            sys.exit(1)

    return client, config


# ============================================================
# CLICK-BASED CLI (if Click is available)
# ============================================================

if HAS_CLICK:

    @click.group()
    @click.version_option(version="1.0.0", prog_name="hytale-cf")
    def main():
        """
        Hytale CurseForge CLI - APT-style mod manager.

        \b
        Examples:
            hytale-cf search magic        Search for mods
            hytale-cf install 12345       Install mod by ID
            hytale-cf list                Show installed mods
        """
        pass

    @main.command()
    @click.argument('query')
    @click.option('-c', '--category', default='mods',
                  type=click.Choice(['mods', 'worlds', 'prefabs', 'bootstrap', 'translations']),
                  help='Category to search')
    @click.option('-n', '--limit', default=10, help='Number of results')
    def search(query: str, category: str, limit: int):
        """Search for mods, worlds, or other resources."""
        _do_search(query, category, limit)

    @main.command()
    @click.argument('mod_id', type=int)
    def info(mod_id: int):
        """Show detailed information about a mod."""
        _do_info(mod_id)

    @main.command()
    @click.argument('mod_id', type=int)
    @click.option('-y', '--yes', is_flag=True, help='Skip confirmation')
    def install(mod_id: int, yes: bool):
        """Install a mod by its ID."""
        _do_install(mod_id, yes)

    @main.command()
    @click.argument('mod_id', type=int)
    @click.option('-y', '--yes', is_flag=True, help='Skip confirmation')
    def remove(mod_id: int, yes: bool):
        """Remove an installed mod."""
        _do_remove(mod_id, yes)

    @main.command('list')
    @click.option('-v', '--verbose', is_flag=True, help='Show detailed info')
    def list_installed(verbose: bool):
        """List all installed mods."""
        _do_list(verbose)

    @main.command()
    @click.option('-y', '--yes', is_flag=True, help='Skip confirmation')
    def update(yes: bool):
        """Check for and install updates."""
        _do_update(yes)

    @main.command()
    @click.option('--api-key', help="Set API key (use single quotes: --api-key 'KEY')")
    @click.option('--api-key-prompt', is_flag=True, help='Set API key interactively (recommended)')
    @click.option('--game-path', help='Set Hytale game directory')
    @click.option('--show', is_flag=True, help='Show current config')
    def config(api_key: str, api_key_prompt: bool, game_path: str, show: bool):
        """Configure API key and game path."""
        if api_key_prompt:
            _do_config_interactive_key()
        else:
            _do_config(api_key, game_path, show)


# ============================================================
# ARGPARSE-BASED CLI (fallback)
# ============================================================

else:
    import argparse

    def main():
        """Main entry point using argparse."""
        parser = argparse.ArgumentParser(
            prog='hytale-cf',
            description='Hytale CurseForge CLI - APT-style mod manager'
        )
        parser.add_argument('--version', action='version', version='hytale-cf 1.0.0')

        subparsers = parser.add_subparsers(dest='command', help='Commands')

        # search
        p_search = subparsers.add_parser('search', help='Search for mods')
        p_search.add_argument('query', help='Search query')
        p_search.add_argument('-c', '--category', default='mods',
                             choices=['mods', 'worlds', 'prefabs', 'bootstrap', 'translations'])
        p_search.add_argument('-n', '--limit', type=int, default=10)

        # info
        p_info = subparsers.add_parser('info', help='Show mod details')
        p_info.add_argument('mod_id', type=int, help='Mod ID')

        # install
        p_install = subparsers.add_parser('install', help='Install a mod')
        p_install.add_argument('mod_id', type=int, help='Mod ID')
        p_install.add_argument('-y', '--yes', action='store_true', help='Skip confirmation')

        # remove
        p_remove = subparsers.add_parser('remove', help='Remove a mod')
        p_remove.add_argument('mod_id', type=int, help='Mod ID')
        p_remove.add_argument('-y', '--yes', action='store_true', help='Skip confirmation')

        # list
        p_list = subparsers.add_parser('list', help='List installed mods')
        p_list.add_argument('-v', '--verbose', action='store_true')

        # update
        p_update = subparsers.add_parser('update', help='Update all mods')
        p_update.add_argument('-y', '--yes', action='store_true')

        # config
        p_config = subparsers.add_parser('config', help='Configure settings')
        p_config.add_argument('--api-key', help="Set API key (use single quotes)")
        p_config.add_argument('--api-key-prompt', action='store_true', help='Set API key interactively (recommended)')
        p_config.add_argument('--game-path', help='Set game path')
        p_config.add_argument('--show', action='store_true', help='Show config')

        args = parser.parse_args()

        if args.command == 'search':
            _do_search(args.query, args.category, args.limit)
        elif args.command == 'info':
            _do_info(args.mod_id)
        elif args.command == 'install':
            _do_install(args.mod_id, args.yes)
        elif args.command == 'remove':
            _do_remove(args.mod_id, args.yes)
        elif args.command == 'list':
            _do_list(args.verbose)
        elif args.command == 'update':
            _do_update(args.yes)
        elif args.command == 'config':
            if args.api_key_prompt:
                _do_config_interactive_key()
            else:
                _do_config(args.api_key, args.game_path, args.show)
        else:
            parser.print_help()


# ============================================================
# SHARED COMMAND IMPLEMENTATIONS
# ============================================================

def _confirm(message: str) -> bool:
    """Ask user for confirmation."""
    if HAS_CLICK:
        import click
        return click.confirm(message)
    else:
        response = input(f"{message} [y/N]: ").strip().lower()
        return response in ('y', 'yes')


def _do_search(query: str, category: str, limit: int):
    """Search implementation."""
    client, config = get_client_and_config()

    with out.status(f"Searching for '{query}'"):
        results, total = client.search(query, category=category, page_size=limit)

    if not results:
        out.warning(f"No results found for '{query}'")
        return

    columns = [
        {'name': 'ID', 'style': 'dim', 'width': 8},
        {'name': 'Name', 'style': 'bold'},
        {'name': 'Author', 'style': 'dim'},
        {'name': 'Downloads', 'justify': 'right'},
        {'name': 'Installed', 'justify': 'center'},
    ]

    rows = []
    for mod in results:
        mod_id = str(mod.get('id', ''))
        name = mod.get('name', 'Unknown')[:40]
        authors = mod.get('authors', [])
        author = authors[0]['name'] if authors else 'Unknown'
        downloads = f"{mod.get('downloadCount', 0):,}"
        installed = "[green]✓[/green]" if config.is_installed(mod.get('id')) else ""
        rows.append([mod_id, name, author, downloads, installed])

    out.table(f"Search Results ({len(results)}/{total})", columns, rows)
    out.print("\nUse 'hytale-cf info <ID>' for details or 'hytale-cf install <ID>' to install")


def _do_info(mod_id: int):
    """Info implementation."""
    client, config = get_client_and_config()

    with out.status("Fetching mod info"):
        try:
            mod = client.get_mod(mod_id)
            files = client.get_files(mod_id)
        except Exception as e:
            out.error(str(e))
            return

    if not mod:
        out.error(f"Mod {mod_id} not found")
        return

    authors = ", ".join(a['name'] for a in mod.get('authors', []))
    categories = ", ".join(c['name'] for c in mod.get('categories', []))
    latest_file = files[0] if files else {}
    installed = config.is_installed(mod_id)

    info_lines = [
        f"[bold]Name:[/bold] {mod.get('name', 'Unknown')}",
        f"[bold]ID:[/bold] {mod_id}",
        f"[bold]Authors:[/bold] {authors}",
        f"[bold]Categories:[/bold] {categories}",
        f"[bold]Downloads:[/bold] {mod.get('downloadCount', 0):,}",
        f"[bold]Latest Version:[/bold] {latest_file.get('displayName', 'N/A')}",
        f"[bold]File Size:[/bold] {latest_file.get('fileLength', 0) / 1024 / 1024:.2f} MB",
        f"[bold]Installed:[/bold] {'Yes' if installed else 'No'}",
    ]

    if mod.get('summary'):
        info_lines.append(f"\n[bold]Summary:[/bold] {mod.get('summary', '')[:200]}")

    out.panel('\n'.join(info_lines), title=mod.get('name', 'Mod Info'))

    if mod.get('links', {}).get('websiteUrl'):
        out.print(f"\nWebsite: {mod['links']['websiteUrl']}")


def _do_install(mod_id: int, skip_confirm: bool):
    """Install implementation."""
    client, config = get_client_and_config()

    if not config.game_path:
        out.error("Game path not set. Run: hytale-cf config --game-path /path/to/hytale")
        return

    if config.is_installed(mod_id):
        out.warning(f"Mod {mod_id} is already installed")
        if not skip_confirm and not _confirm("Reinstall?"):
            return

    with out.status("Fetching mod info"):
        try:
            mod = client.get_mod(mod_id)
            latest = client.get_latest_file(mod_id)
        except Exception as e:
            out.error(str(e))
            return

    mod_name = mod.get('name', f'Mod {mod_id}')
    file_name = latest.get('fileName', 'unknown')
    file_size = latest.get('fileLength', 0) / 1024 / 1024

    out.print(f"\n[bold]Package:[/bold] {mod_name}")
    out.print(f"[bold]File:[/bold] {file_name} ({file_size:.2f} MB)")

    if not skip_confirm and not _confirm("\nProceed with installation?"):
        out.warning("Aborted.")
        return

    with out.progress_download(f"Installing {mod_name}...") as progress:
        try:
            result = client.install_mod(mod_id, config.game_path, progress.update)
            config.add_installed(mod_id, result)
        except Exception as e:
            out.error(f"Installation failed: {e}")
            return

    out.success(f"Successfully installed [bold]{mod_name}[/bold]")
    out.print(f"Location: {result['path']}")


def _do_remove(mod_id: int, skip_confirm: bool):
    """Remove implementation."""
    config = Config()

    if not config.game_path:
        out.error("Game path not set.")
        return

    if not config.is_installed(mod_id):
        out.warning(f"Mod {mod_id} is not installed")
        return

    mod_info = config.installed_mods.get(str(mod_id), {})
    mod_name = mod_info.get('name', f'Mod {mod_id}')

    if not skip_confirm and not _confirm(f"Remove {mod_name}?"):
        out.warning("Aborted.")
        return

    with out.status(f"Removing {mod_name}"):
        try:
            client, _ = get_client_and_config()
            success = client.uninstall_mod(mod_info, config.game_path)
            if success:
                config.remove_installed(mod_id)
                out.success(f"Removed {mod_name}")
            else:
                out.warning("File not found, removing from tracking")
                config.remove_installed(mod_id)
        except Exception as e:
            out.error(str(e))


def _do_list(verbose: bool):
    """List implementation."""
    config = Config()
    installed = config.installed_mods

    if not installed:
        out.print("No mods installed.")
        out.print("Use 'hytale-cf search <query>' to find mods")
        return

    columns = [
        {'name': 'ID', 'style': 'dim', 'width': 8},
        {'name': 'Name', 'style': 'bold'},
        {'name': 'Version'},
    ]
    if verbose:
        columns.append({'name': 'File'})
        columns.append({'name': 'Path'})

    rows = []
    for mod_id, info in installed.items():
        row = [
            mod_id,
            info.get('name', 'Unknown')[:35],
            info.get('version', 'N/A')[:20],
        ]
        if verbose:
            row.append(info.get('filename', 'N/A'))
            row.append(info.get('path', 'N/A')[:40])
        rows.append(row)

    out.table(f"Installed Mods ({len(installed)})", columns, rows)


def _do_update(skip_confirm: bool):
    """Update implementation."""
    client, config = get_client_and_config()

    installed = config.installed_mods
    if not installed:
        out.print("No mods installed.")
        return

    out.print(f"Checking {len(installed)} mods for updates...\n")

    updates_available = []

    with out.status("Checking for updates") as status:
        for mod_id, info in installed.items():
            try:
                latest = client.get_latest_file(int(mod_id))
                current_file_id = info.get('file_id')
                if latest.get('id') != current_file_id:
                    updates_available.append({
                        'mod_id': int(mod_id),
                        'name': info.get('name', mod_id),
                        'current': info.get('version', 'N/A'),
                        'new': latest.get('displayName', 'N/A'),
                    })
            except Exception:
                pass

    if not updates_available:
        out.success("All mods are up to date!")
        return

    columns = [
        {'name': 'ID', 'style': 'dim'},
        {'name': 'Name', 'style': 'bold'},
        {'name': 'Current'},
        {'name': 'New', 'style': 'green'},
    ]

    rows = [[str(u['mod_id']), u['name'], u['current'], u['new']] for u in updates_available]
    out.table("Updates Available", columns, rows)

    if not skip_confirm and not _confirm(f"\nUpdate {len(updates_available)} mods?"):
        out.warning("Aborted.")
        return

    for upd in updates_available:
        out.print(f"\nUpdating {upd['name']}...")
        try:
            result = client.install_mod(upd['mod_id'], config.game_path)
            config.add_installed(upd['mod_id'], result)
            out.success(f"Updated {upd['name']}")
        except Exception as e:
            out.error(f"Failed to update {upd['name']}: {e}")


def _get_input(prompt: str) -> str:
    """Get input from user, works with getpass for sensitive data."""
    import getpass
    try:
        return getpass.getpass(prompt)
    except Exception:
        return input(prompt)


def _do_config(api_key: str, game_path: str, show: bool):
    """Config implementation."""
    cfg = Config()

    if show:
        key_display = ('*' * 8 + cfg.api_key[-8:]) if cfg.api_key else 'Not set'
        content = f"[bold]API Key:[/bold] {key_display}\n"
        content += f"[bold]Game Path:[/bold] {cfg.game_path or 'Not set'}\n"
        content += f"[bold]Config File:[/bold] {cfg.config_path}"
        out.panel(content, title="Current Configuration")
        return

    if api_key:
        # If api_key is a flag without value or looks corrupted, prompt interactively
        if api_key == 'True' or len(api_key) < 20 or not api_key.startswith('$'):
            # Looks like a valid key passed correctly
            cfg.api_key = api_key
            out.success("API key saved")
        else:
            cfg.api_key = api_key
            out.success("API key saved")

    if game_path:
        import os
        if os.path.isdir(game_path):
            cfg.game_path = game_path
            out.success(f"Game path set to: {cfg.game_path}")
        else:
            out.error(f"Directory does not exist: {game_path}")
            return

    if not api_key and not game_path:
        out.print("Usage:")
        out.print("  hytale-cf config --api-key 'YOUR_KEY'  (use single quotes!)")
        out.print("  hytale-cf config --api-key-prompt      (interactive, safer)")
        out.print("  hytale-cf config --game-path /path/to/hytale")
        out.print("  hytale-cf config --show")


def _do_config_interactive_key():
    """Set API key interactively to avoid shell escaping issues."""
    cfg = Config()
    out.print("Enter your CurseForge API key (input is hidden):")
    api_key = _get_input("API Key: ")
    if api_key and len(api_key) > 10:
        cfg.api_key = api_key.strip()
        out.success("API key saved")
    else:
        out.error("Invalid API key")


if __name__ == '__main__':
    main()
