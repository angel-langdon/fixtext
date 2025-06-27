import os
try:
    from msvcrt import getch as windows_getch
    getch = lambda: windows_getch().decode()
except ImportError:
    from getch import getch

import pyperclip
from dotenv import load_dotenv
from google import genai
from google.genai import types

from formats import formats

load_dotenv()


GEMINI_API_KEY = os.environ["GEMINI_API_KEY"]


def gemini_completion(prompt: str, system_instructions: str):
    client = genai.Client(
        api_key=GEMINI_API_KEY,
    )

    model = "gemini-2.5-flash"
    contents = [types.Content(role="user", parts=[types.Part.from_text(text=prompt)])]
    generate_content_config = types.GenerateContentConfig(
        response_mime_type="text/plain",
        system_instruction=[types.Part.from_text(text=system_instructions)],
        thinking_config=types.ThinkingConfig(thinking_budget=0),
    )

    chunks: list[str] = []
    for chunk in client.models.generate_content_stream(
        model=model,
        contents=contents,
        config=generate_content_config,
    ):
        if chunk.text is None:
            continue
        chunks.append(chunk.text)
        print(chunk.text, end="")
    return "".join(chunks)


msg = "\n".join(f"{i}. {f.title}" for i, f in enumerate(formats)) + "\n"

if __name__ == "__main__":
    print(msg)
    print("Number format to use: ")
    n = getch()
    print(n)
    text = pyperclip.paste()
    i = int(n)
    f = formats[i]
    modified_text = gemini_completion(text, f.system_instructions)
    pyperclip.copy(modified_text)
