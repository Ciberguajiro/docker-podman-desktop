import os
try:
    content = open('src/routes/volumes/+page.svelte').read()
    print("---CONTENT_START---")
    print(content)
    print("---CONTENT_END---")
except Exception as e:
    print(f"Error: {e}")
