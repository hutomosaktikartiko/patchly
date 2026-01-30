async function getOPFSRoot(): Promise<FileSystemDirectoryHandle> {
  return await navigator.storage.getDirectory();
}

async function createOPFSFile(
  filename: string,
): Promise<FileSystemWritableFileStream> {
  const root = await getOPFSRoot();
  const fileHandle = await root.getFileHandle(filename, {create: true});

  return await fileHandle.createWritable();
}

async function readOPFSFile(filename: string): Promise<File> {
  const root = await getOPFSRoot();
  const fileHandle = await root.getFileHandle(filename);
  
  return await fileHandle.getFile();
}

async function deleteOPFSFile(filename: string): Promise<void> {
  const root = await getOPFSRoot();

  return root.removeEntry(filename);
}

function isOPFSAvailable(): boolean {
  return 'storage' in navigator && 'getDirectory' in navigator.storage;
}

export { getOPFSRoot, createOPFSFile, readOPFSFile, deleteOPFSFile, isOPFSAvailable };