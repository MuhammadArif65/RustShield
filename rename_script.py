import os
import subprocess

def replace_in_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        return

    new_content = content.replace("ferrumward", "ferrumward") \
                         .replace("FerrumWard", "FerrumWard") \
                         .replace("FERRUMWARD", "FERRUMWARD") \
                         .replace("Ferrumward", "Ferrumward") \
                         .replace("ferrumWard", "ferrumWard")

    if content != new_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)

def git_mv(src, dst):
    if os.path.exists(src):
        subprocess.run(["git", "mv", src, dst])

# 1. Search and replace in all files
for root, dirs, files in os.walk('.'):
    if '.git' in dirs:
        dirs.remove('.git')
    if 'target' in dirs:
        dirs.remove('target')

    for file in files:
        filepath = os.path.join(root, file)
        replace_in_file(filepath)

# 2. Rename specific files with FerrumWard in name
files_to_rename = [
    ("ferrumward-ffi/include/ferrumward.h", "ferrumward-ffi/include/ferrumward.h"),
    ("integrations/unity/FerrumWard.cs", "integrations/unity/FerrumWard.cs"),
    ("integrations/unreal/FerrumWard.h", "integrations/unreal/FerrumWard.h")
]

for src, dst in files_to_rename:
    git_mv(src, dst)

# 3. Rename top-level directories
dirs = [d for d in os.listdir('.') if os.path.isdir(d) and d.startswith('ferrumward-')]
for d in dirs:
    new_d = d.replace('ferrumward', 'ferrumward')
    git_mv(d, new_d)

print("Renaming completed successfully.")
