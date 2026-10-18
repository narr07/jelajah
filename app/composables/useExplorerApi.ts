import { convertFileSrc, invoke as tauriInvoke } from '@tauri-apps/api/core';

// Rust returns Err(String); wrap it so callers' `e.message` works.
const invoke = <T>(cmd: string, args?: Record<string, unknown>) =>
	tauriInvoke<T>(cmd, args).catch((e) => {
		throw e instanceof Error ? e : new Error(String(e));
	});

/** Local file -> URL the webview can stream (asset protocol). */
export const fileSrc = (path: string) => convertFileSrc(path);

export interface FileItem {
	name: string;
	path: string;
	is_dir: boolean;
	size: number;
	modified: number;
	created: number;
	readonly: boolean;
	hidden: boolean;
	extension: string;
}

export interface DriveItem {
	letter: string;
	path: string;
	label: string;
	total_bytes?: number;
	free_bytes?: number;
}

export interface QuickAccessItem {
	id: string;
	name: string;
	path: string;
	icon: string;
}

export interface SearchHit {
	path: string;
	name: string;
	is_dir: boolean;
	line_number?: number;
	line_content?: string;
	match_start?: number;
	match_end?: number;
}

export interface FilePreview {
	is_dir: boolean;
	is_image?: boolean;
	is_binary?: boolean;
	size: number;
	modified: number;
	created: number;
	preview: string | null;
	media?: 'image' | 'pdf' | 'video' | 'audio' | null;
}

export interface TreeSizeResult {
	targetPath: string;
	totalSize: number;
	totalFiles: number;
	totalFolders: number;
	subfolders: {
		name: string;
		path: string;
		size: number;
		percent: number;
		filesCount: number;
		dirsCount: number;
	}[];
	byCategory: {
		category: string;
		size: number;
		count: number;
		percent: number;
	}[];
	topLargestFiles: {
		name: string;
		path: string;
		size: number;
		ext: string;
	}[];
}

export interface FileHashResult {
	name: string;
	path: string;
	size: number;
	modified: number;
	md5: string;
	sha1: string;
	sha256: string;
	sha512: string;
}

export interface GitStatusResult {
	isRepo: boolean;
	repoRoot?: string;
	branch?: string;
	ahead?: number;
	behind?: number;
	fileStatuses?: Record<string, string>;
}

// Nuxt is UI only: every call goes to the Rust backend via Tauri IPC.
export const useExplorerApi = () => {
	const listDirectory = (targetPath: string, showHidden = false) =>
		invoke<{ current_path: string; items: FileItem[] }>('list_directory', { path: targetPath, showHidden });

	const getDrives = async () => (await invoke<{ drives: DriveItem[] }>('get_drives')).drives;

	const getQuickAccess = async () => (await invoke<{ items: QuickAccessItem[] }>('get_quick_access')).items;

	const runTgrep = async (query: string, searchPath: string, contentSearch = false, caseSensitive = false, regex = false) =>
		(await invoke<{ hits: SearchHit[]; count: number }>('run_tgrep', { query, searchPath, contentSearch, caseSensitive, regex })).hits;

	/** Everything-style name index (all fixed drives). null = still building / folder not indexed. */
	const searchNames = (query: string, within?: string) =>
		invoke<SearchHit[] | null>('search_names', { query, within });

	const buildTgrepIndex = (targetPath: string) =>
		invoke<{ success: boolean; message: string }>('build_tgrep_index', { path: targetPath });

	const executeAction = (
		action: 'open' | 'openTerminal' | 'createFolder' | 'createFile' | 'rename' | 'delete' | 'deletePermanent' | 'copy' | 'move',
		targetPath: string,
		options?: { name?: string; newPath?: string; destinationPath?: string; content?: string; contentBase64?: string }
	) => invoke<any>('file_action', { action, targetPath, ...options });

	/** inline=true returns images as a data URL (needed for canvas work like Image Resizer). */
	const getPreview = (targetPath: string, inline = false) => invoke<FilePreview>('get_file_preview', { path: targetPath, inline });

	/** First call scans; later calls for folders inside that scan answer from the Rust-side cache. */
	const getTreeSize = (path: string, rescan = false) => invoke<TreeSizeResult>('fast_treesize', { path, rescan });

	const searchTgrepSidecar = (root: string, query: string) =>
		invoke<{ path: string; line: number; text: string }[]>('search', { root, query });

	const getSearchStatus = (root: string) => invoke<string>('search_status', { root });

	const getFileHash = (path: string) => invoke<FileHashResult>('fast_file_hash', { path });

	const getGitStatus = (path: string) => invoke<GitStatusResult>('get_git_status', { path });

	const getFileLocksmith = (path: string) =>
		invoke<{ isLocked: boolean; path: string; processes: string[] }>('file_locksmith', { path });

	// ---- PowerToys-style tools (src-tauri/src/tools.rs) ----
	const newplusList = () => invoke<{ dir: string; items: { name: string; is_dir: boolean }[] }>('newplus_list');
	const newplusCreate = (template: string, destDir: string) => invoke<string>('newplus_create', { template, destDir });
	const ocrImage = (path: string) => invoke<string>('ocr_image', { path });
	const fileIcon = (path: string, isDir: boolean) => invoke<string | null>('file_icon', { path, isDir });
	/** Explorer's own context menu at the cursor ("Show more options"). */
	const shellContextMenu = (path: string) => invoke<void>('shell_context_menu', { path });
	const showProperties = (path: string) => invoke<void>('show_properties', { path });

	// ---- Elevated MFT helper for the name index (src-tauri/src/fast_index.rs) ----
	const fastIndexStatus = () => invoke<boolean>('fast_index_status');
	const setFastIndex = (enable: boolean) => invoke<void>('set_fast_index', { enable });

	return {
		newplusList,
		newplusCreate,
		ocrImage,
		fileIcon,
		shellContextMenu,
		showProperties,
		fastIndexStatus,
		setFastIndex,
		listDirectory,
		getDrives,
		getQuickAccess,
		runTgrep,
		searchNames,
		buildTgrepIndex,
		executeAction,
		getPreview,
		getTreeSize,
		getFileHash,
		getGitStatus,
		getFileLocksmith,
		searchTgrepSidecar,
		getSearchStatus
	};
};
