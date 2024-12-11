export const asyncFileClick = async (
  fileInputRef2: React.RefObject<HTMLInputElement>
): Promise<File | undefined> => {
  await fileInputRef2.current?.click();
  return new Promise<File | undefined>((resolve, reject) => {
    if (!fileInputRef2.current) {
      reject(new Error("File input reference is not defined."));
      return;
    }

    const input = fileInputRef2.current;

    const handleChange = (event: Event) => {
      input.removeEventListener("change", handleChange);

      const target = event.target as HTMLInputElement;
      const file = target.files?.[0];
      resolve(file); // 파일 반환
    };

    input.addEventListener("change", handleChange);
  });
};