import type { ChatMessage } from "@/pages/assistant"

const PAGE_WIDTH = 595.28
const PAGE_HEIGHT = 841.89
const MARGIN = 48
const TEXT_WIDTH = PAGE_WIDTH - MARGIN * 2

type PdfFont = "F1" | "F2"

interface PdfLine {
  text: string
  font: PdfFont
  size: number
  x: number
  y: number
}

function normalizeTranscriptText(input: string) {
  const normalized = input
    .replace(/\r\n/g, "\n")
    .replace(/```[\s\S]*?```/g, (match) => match.replace(/```/g, ""))
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/\[(.*?)\]\((.*?)\)/g, "$1 ($2)")
    .replace(/^\s*[-*+]\s+/gm, "- ")
    .replace(/^\s*(\d+)\.\s+/gm, "$1. ")
    .replace(/\n{3,}/g, "\n\n")
    .trim()
    .normalize("NFKD")

  return normalized
    .split("\n")
    .map((line) => line.replace(/[^\x20-\x7E]/g, ""))
    .join("\n")
}

function escapePdfText(text: string) {
  return text.replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)")
}

function estimateMaxChars(size: number, indent = 0) {
  return Math.max(16, Math.floor((TEXT_WIDTH - indent) / (size * 0.52)))
}

function wrapParagraph(paragraph: string, maxChars: number) {
  if (!paragraph) return [""]

  const words = paragraph.split(/\s+/)
  const lines: string[] = []
  let current = ""

  const pushCurrent = () => {
    if (current) {
      lines.push(current)
      current = ""
    }
  }

  for (const word of words) {
    if (!word) continue

    if (!current) {
      current = word
      continue
    }

    if ((current + " " + word).length <= maxChars) {
      current += ` ${word}`
      continue
    }

    pushCurrent()

    if (word.length <= maxChars) {
      current = word
      continue
    }

    let chunk = ""
    for (const char of word) {
      if ((chunk + char).length > maxChars) {
        lines.push(chunk)
        chunk = char
      } else {
        chunk += char
      }
    }
    current = chunk
  }

  pushCurrent()
  return lines.length > 0 ? lines : [paragraph]
}

function pushLine(pages: PdfLine[][], line: Omit<PdfLine, "y">, cursorY: number) {
  const page = pages[pages.length - 1]
  const lineHeight = line.size * 1.35

  if (cursorY - lineHeight < MARGIN) {
    pages.push([])
    return pushLine(pages, line, PAGE_HEIGHT - MARGIN)
  }

  page.push({ ...line, y: cursorY })
  return cursorY - lineHeight
}

function addWrappedText(
  pages: PdfLine[][],
  text: string,
  options: { font: PdfFont; size: number; x: number; cursorY: number; indent?: number },
) {
  const { font, size, x, indent = 0 } = options
  let cursorY = options.cursorY
  const maxChars = estimateMaxChars(size, indent)
  const paragraphs = text.split("\n")

  for (let index = 0; index < paragraphs.length; index += 1) {
    const paragraph = paragraphs[index]
    if (paragraph.trim() === "") {
      cursorY -= size * 0.8
      continue
    }

    for (const wrappedLine of wrapParagraph(paragraph, maxChars)) {
      cursorY = pushLine(pages, { text: wrappedLine, font, size, x: x + indent }, cursorY)
    }

    if (index < paragraphs.length - 1) {
      cursorY -= size * 0.5
    }
  }

  return cursorY
}

function layoutTranscript(messages: ChatMessage[]) {
  const pages: PdfLine[][] = [[]]
  let cursorY = PAGE_HEIGHT - MARGIN

  cursorY = addWrappedText(pages, "LiveStack Assistant Transcript", {
    font: "F2",
    size: 18,
    x: MARGIN,
    cursorY,
  })
  cursorY -= 6
  cursorY = addWrappedText(
    pages,
    `Exported ${new Date().toLocaleString([], {
      dateStyle: "medium",
      timeStyle: "short",
    })}`,
    {
      font: "F1",
      size: 10,
      x: MARGIN,
      cursorY,
    },
  )
  cursorY -= 14

  if (messages.length === 0) {
    addWrappedText(pages, "No messages yet.", {
      font: "F1",
      size: 11,
      x: MARGIN,
      cursorY,
    })
  } else {
    for (const message of messages) {
      cursorY -= 2
      cursorY = addWrappedText(pages, message.role === "user" ? "User" : "Assistant", {
        font: "F2",
        size: 12,
        x: MARGIN,
        cursorY,
      })
      cursorY = addWrappedText(pages, normalizeTranscriptText(message.content), {
        font: "F1",
        size: 11,
        x: MARGIN,
        cursorY: cursorY - 4,
        indent: 12,
      })
      cursorY -= 10
    }
  }

  return pages
}

function buildPdfObjectStream(content: string) {
  return `<< /Length ${content.length} >>\nstream\n${content}\nendstream`
}

function buildContentStream(lines: PdfLine[], pageNumber: number, totalPages: number) {
  const parts: string[] = []

  for (const line of lines) {
    parts.push(
      `BT /${line.font} ${line.size.toFixed(1)} Tf 1 0 0 1 ${line.x.toFixed(2)} ${line.y.toFixed(
        2,
      )} Tm (${escapePdfText(line.text)}) Tj ET`,
    )
  }

  parts.push(
    `BT /F1 9 Tf 1 0 0 1 ${MARGIN.toFixed(2)} ${24} Tm (Page ${pageNumber} of ${totalPages}) Tj ET`,
  )

  return parts.join("\n")
}

function buildPdfDocument(pages: PdfLine[][]) {
  const objects = new Map<number, string>()
  const offsets: number[] = []

  const catalogId = 1
  const pagesId = 2
  const regularFontId = 3
  const boldFontId = 4
  let nextId = 5

  const pageObjectIds: number[] = []

  for (let index = 0; index < pages.length; index += 1) {
    const page = pages[index]
    const contentId = nextId++
    const pageId = nextId++
    pageObjectIds.push(pageId)
    objects.set(contentId, buildPdfObjectStream(buildContentStream(page, index + 1, pages.length)))
    objects.set(
      pageId,
      `<< /Type /Page /Parent ${pagesId} 0 R /MediaBox [0 0 ${PAGE_WIDTH.toFixed(
        2,
      )} ${PAGE_HEIGHT.toFixed(2)}] /Resources << /Font << /F1 ${regularFontId} 0 R /F2 ${boldFontId} 0 R >> >> /Contents ${contentId} 0 R >>`,
    )
  }

  objects.set(catalogId, `<< /Type /Catalog /Pages ${pagesId} 0 R >>`)
  objects.set(
    pagesId,
    `<< /Type /Pages /Count ${pageObjectIds.length} /Kids [${pageObjectIds
      .map((id) => `${id} 0 R`)
      .join(" ")}] >>`,
  )
  objects.set(regularFontId, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
  objects.set(boldFontId, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>")

  let pdf = "%PDF-1.4\n"
  const objectCount = boldFontId + 2 * pages.length
  for (let id = 1; id <= objectCount; id += 1) {
    const object = objects.get(id)
    if (!object) {
      throw new Error(`Missing PDF object ${id}`)
    }
    offsets.push(pdf.length)
    pdf += `${id} 0 obj\n${object}\nendobj\n`
  }

  const xrefStart = pdf.length
  pdf += `xref\n0 ${objectCount + 1}\n`
  pdf += "0000000000 65535 f \n"
  for (const offset of offsets) {
    pdf += `${offset.toString().padStart(10, "0")} 00000 n \n`
  }
  pdf += `trailer\n<< /Size ${objectCount + 1} /Root ${catalogId} 0 R >>\nstartxref\n${xrefStart}\n%%EOF`

  return pdf
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = filename
  anchor.rel = "noreferrer"
  document.body.appendChild(anchor)
  anchor.click()
  anchor.remove()
  window.setTimeout(() => URL.revokeObjectURL(url), 1000)
}

export function exportAssistantTranscriptPdf(messages: ChatMessage[]) {
  const pages = layoutTranscript(messages)
  const pdf = buildPdfDocument(pages)
  const blob = new Blob([pdf], { type: "application/pdf" })
  const stamp = new Date().toISOString().slice(0, 10)
  downloadBlob(blob, `livestack-assistant-transcript-${stamp}.pdf`)
}
