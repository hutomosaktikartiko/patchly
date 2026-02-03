async function getOpfsRoot(): Promise<FileSystemDirectoryHandle> {
  return await navigator.storage.getDirectory();
}

async function createOpfsFile(
  filename: string,
): Promise<FileSystemWritableFileStream> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, {create: true});

  return await fileHandle.createWritable();
}

async function getOpfsFile(filename: string): Promise<File> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename);
  
  return await fileHandle.getFile();
}

async function deleteOpfsFile(filename: string): Promise<void> {
  const root = await getOpfsRoot();

  return root.removeEntry(filename);
}

function isOpfsAvailable(): boolean {
  return 'storage' in navigator && 'getDirectory' in navigator.storage;
}

export { getOpfsRoot, createOpfsFile, getOpfsFile, deleteOpfsFile, isOpfsAvailable };