# UI Plan: Slick Sheet Studio

## Current State Analysis

### What Works
- Split-pane editor (code left, preview right)
- Template gallery via "New" button (10 templates)
- Save project as JSON
- Load project from JSON
- Export PDF
- Auto-preview toggle with debounce
- Manual refresh

### What's Missing
1. **AI Features not exposed in UI**
   - No chat panel for AI prompts
   - No settings modal for API key
   - No model selector
   - No AI progress indicator

2. **Click-to-edit not functional**
   - `cmd://edit/` links exist in Typst code
   - Clicking in preview doesn't open edit modals
   - Link interception code exists but not connected

---

## Proposed UI Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Slick Sheet Studio          [New] [Open] [Save] [Export PDF] [⚙️ Settings] │
│  ─────────────────────────────────────────────────────────────────────── │
│  [Auto-preview ✓] [Refresh]                                              │
├────────────────────────┬────────────────────────┬───────────────────────┤
│                        │                        │                       │
│      CODE EDITOR       │       PREVIEW          │      AI CHAT          │
│                        │                        │                       │
│  ┌──────────────────┐  │  ┌──────────────────┐  │  ┌─────────────────┐  │
│  │ #set page(...)   │  │  │                  │  │  │ Chat history    │  │
│  │ #set text(...)   │  │  │   [Rendered      │  │  │                 │  │
│  │                  │  │  │    Document]     │  │  │ User: Make the  │  │
│  │ = Title          │  │  │                  │  │  │ title red       │  │
│  │                  │  │  │   Click any      │  │  │                 │  │
│  │ Body text...     │  │  │   element to     │  │  │ AI: [Working...│  │
│  │                  │  │  │   edit           │  │  │                 │  │
│  │                  │  │  │                  │  │  ├─────────────────┤  │
│  │                  │  │  │                  │  │  │ [Type prompt..] │  │
│  │                  │  │  │                  │  │  │         [Send]  │  │
│  └──────────────────┘  │  └──────────────────┘  │  └─────────────────┘  │
│                        │                        │                       │
├────────────────────────┴────────────────────────┴───────────────────────┤
│  Status: Ready                                            [Offline: ⚠️]  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Component Breakdown

### 1. Toolbar (Enhanced)

**Current:**
```
[New] [Open] [Save] [Export PDF] | [Auto-preview ✓] [Refresh]
```

**Proposed:**
```
[New] [Open] [Save] [Export PDF] | [Auto-preview ✓] [Refresh] | [⚙️ Settings]
```

**New Elements:**
- **Settings button (⚙️)**: Opens AI settings modal

---

### 2. AI Settings Modal

Opens when clicking Settings button.

```
┌─────────────────────────────────────────────┐
│  AI Settings                           [×]  │
├─────────────────────────────────────────────┤
│                                             │
│  API Key (OpenRouter)                       │
│  ┌─────────────────────────────────────┐    │
│  │ ●●●●●●●●●●●●●●●●                    │    │
│  └─────────────────────────────────────┘    │
│  Get key: https://openrouter.ai/keys        │
│                                             │
│  Model                                      │
│  ┌─────────────────────────────────────┐    │
│  │ Gemini 3 Flash (Fast & Cheap)    ▼  │    │
│  └─────────────────────────────────────┘    │
│    • Gemini 3 Flash (Fast & Cheap)          │
│    • Claude 4.5 Haiku (Balanced)            │
│    • GPT-5.2 Mini (Alternative)             │
│    • Claude Sonnet 4 (Best Quality)         │
│                                             │
│  Max Iterations                             │
│  ┌────┐                                     │
│  │ 3  │ ──●────────── (1-10)                │
│  └────┘                                     │
│                                             │
│  [Cancel]                        [Save]     │
└─────────────────────────────────────────────┘
```

**Behavior:**
- API key stored in localStorage (encrypted/obfuscated)
- Model selection persisted
- Max iterations for agent loop (default: 3)

---

### 3. AI Chat Panel (New Component)

Collapsible panel on the right side of the editor.

```
┌─────────────────────────────────┐
│  AI Assistant            [−]    │
├─────────────────────────────────┤
│                                 │
│  ┌─────────────────────────┐    │
│  │ 💬 Chat History         │    │
│  │                         │    │
│  │ You: Make the title red │    │
│  │                         │    │
│  │ AI: I'll update the     │    │
│  │ title color to red.     │    │
│  │ [Iteration 1/3]         │    │
│  │ ✓ Generated code        │    │
│  │ ✓ Compiled successfully │    │
│  │ ⏳ Verifying...         │    │
│  │                         │    │
│  └─────────────────────────┘    │
│                                 │
├─────────────────────────────────┤
│  ┌─────────────────────┐ [Send] │
│  │ Type your request...│        │
│  └─────────────────────┘        │
│                                 │
│  [⚠️ Configure API key first]   │
└─────────────────────────────────┘
```

**States:**
1. **No API key**: Shows warning, Send disabled
2. **Ready**: Input enabled, Send enabled
3. **Processing**: Shows progress, input disabled
4. **Error**: Shows error message, retry option

**Features:**
- Chat history with user/AI messages
- Progress indicator during agent loop
- Iteration counter (e.g., "1/3")
- Step-by-step status:
  - ⏳ Generating code...
  - ✓ Generated code
  - ⏳ Compiling...
  - ✓ Compiled successfully
  - ⏳ Verifying with vision...
  - ✓ Verification passed / ✗ Needs retry

---

### 4. Click-to-Edit Modal (Fix Existing)

The code already has `cmd://edit/` URLs in the Typst templates. Need to:
1. Intercept clicks on links in the SVG preview
2. Parse the `cmd://edit/{field}` URL
3. Open edit modal for that field

```
┌─────────────────────────────────────────────┐
│  Edit Title                            [×]  │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────────────────────────────┐    │
│  │ Hello World                         │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  [Cancel]                       [Save]      │
└─────────────────────────────────────────────┘
```

**Editable Fields:**
- `cmd://edit/title` → Edit title
- `cmd://edit/subtitle` → Edit subtitle
- `cmd://edit/body` → Edit body (multiline)
- `cmd://edit/image` → Edit image URL
- `cmd://edit/metadata/{key}` → Edit metadata field

---

### 5. Template Gallery (Enhance Existing)

Current gallery works but could be improved:

```
┌─────────────────────────────────────────────────────────────┐
│  Choose a Template                                     [×]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [All] [Marketing] [Business] [Event] [Data] [Minimal]      │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │ [Preview]   │  │ [Preview]   │  │ [Preview]   │          │
│  │             │  │             │  │             │          │
│  │ Product     │  │ Event       │  │ One-Pager   │          │
│  │ Sheet       │  │ Flyer       │  │             │          │
│  │ MARKETING   │  │ EVENT       │  │ MARKETING   │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │ [Preview]   │  │ [Preview]   │  │ [Preview]   │          │
│  │             │  │             │  │             │          │
│  │ Comparison  │  │ Case Study  │  │ Team        │          │
│  │ Chart       │  │             │  │ Profile     │          │
│  │ DATA        │  │ BUSINESS    │  │ BUSINESS    │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│                                                             │
│  [Start Blank]                                              │
└─────────────────────────────────────────────────────────────┘
```

**Enhancements:**
- Category filter tabs
- Live preview thumbnails (render SVG mini)
- "Start Blank" option
- Hover to see larger preview

---

### 6. Status Bar (New)

Bottom bar showing app status:

```
┌─────────────────────────────────────────────────────────────┐
│  Status: Ready  |  Project: Untitled  |  Last saved: Never  │  [🟢 Online]
└─────────────────────────────────────────────────────────────┘
```

**States:**
- 🟢 Online - AI available
- 🔴 Offline - AI unavailable (show warning)
- ⏳ Compiling...
- ✓ Compiled
- ⏳ AI processing...
- ✓ AI complete

---

## User Flows

### Flow 1: New User Getting Started

```
1. User opens app
   └─→ See default "Hello World" document

2. User clicks "New"
   └─→ Template gallery opens

3. User selects "Product Sheet"
   └─→ Template loads in editor
   └─→ Preview shows rendered document

4. User clicks in preview on "Product Name"
   └─→ Edit modal opens
   └─→ User types new name
   └─→ Clicks Save
   └─→ Document updates
```

### Flow 2: Using AI to Modify Document

```
1. User has a document loaded

2. User clicks Settings (⚙️)
   └─→ Settings modal opens

3. User enters API key, selects model
   └─→ Clicks Save
   └─→ AI panel shows "Ready"

4. User types in AI chat: "Make the title larger and red"
   └─→ Clicks Send

5. AI processes:
   └─→ "Generating code..."
   └─→ "Compiling..."
   └─→ "Verifying..."
   └─→ Success: Code updates in editor
   └─→ Preview refreshes automatically

6. If verification fails:
   └─→ AI retries with feedback
   └─→ Shows iteration counter
   └─→ After max iterations: shows error
```

### Flow 3: Save and Export

```
1. User finishes editing

2. User clicks "Save"
   └─→ Browser downloads "Project Name.json"
   └─→ Status bar: "Project saved!"

3. User clicks "Export PDF"
   └─→ Browser downloads "Project Name.pdf"
   └─→ Status bar: "PDF exported!"

4. Later, user clicks "Open"
   └─→ File picker opens
   └─→ User selects .json file
   └─→ Project loads
   └─→ Status bar: "Project loaded!"
```

### Flow 4: Offline Usage

```
1. User loses internet connection
   └─→ Status bar shows: 🔴 Offline
   └─→ AI chat shows: "AI unavailable offline"

2. User can still:
   └─→ Edit Typst code
   └─→ See live preview
   └─→ Save/load projects
   └─→ Export PDF

3. User goes back online
   └─→ Status bar shows: 🟢 Online
   └─→ AI features re-enabled
```

---

## Implementation Priority

### Phase A: Fix Click-to-Edit (High Priority)
1. Add click event listener to preview pane
2. Detect `cmd://edit/` links in SVG
3. Show edit modal with current value
4. Update Content struct on save
5. Regenerate Typst and refresh preview

### Phase B: Add AI Settings Modal (High Priority)
1. Create SettingsModal component
2. Add localStorage persistence for API key
3. Add Settings button to toolbar
4. Wire up OpenRouterConfig

### Phase C: Add AI Chat Panel (High Priority)
1. Create ChatPanel component
2. Add to editor layout (right side, collapsible)
3. Connect to AgentLoop
4. Show progress during processing
5. Update editor on success

### Phase D: Enhance Template Gallery (Medium Priority)
1. Add category filter tabs
2. Add preview thumbnails
3. Add "Start Blank" option

### Phase E: Add Status Bar (Low Priority)
1. Create StatusBar component
2. Show online/offline status
3. Show project info
4. Show last saved time

---

## File Changes Required

### New Files
- `src/editor/settings_modal.rs` - AI settings modal
- `src/editor/chat_panel.rs` - AI chat interface
- `src/editor/edit_modal.rs` - Click-to-edit modal
- `src/editor/status_bar.rs` - Bottom status bar

### Modified Files
- `src/editor/mod.rs` - Add new components, wire up AI
- `src/editor/preview.rs` - Add click handler for cmd:// links
- `src/main.rs` - No changes needed

### Component Tree
```
Editor
├── Toolbar
│   ├── AppTitle
│   ├── FileButtons (New, Open, Save, Export PDF)
│   ├── PreviewToggle
│   └── SettingsButton (NEW)
├── MainContent
│   ├── CodePane
│   │   └── CodeEditor
│   ├── PreviewPane (enhanced with click handling)
│   │   └── Preview
│   └── ChatPanel (NEW, collapsible)
│       ├── ChatHistory
│       └── ChatInput
├── Modals
│   ├── TemplateGalleryModal
│   ├── SettingsModal (NEW)
│   └── EditModal (NEW)
└── StatusBar (NEW)
```

---

## Testing Checklist

### Click-to-Edit
- [ ] Click on title in preview opens edit modal
- [ ] Edit modal shows current value
- [ ] Save updates the document
- [ ] Cancel closes without changes
- [ ] Works for all field types

### AI Settings
- [ ] Settings button opens modal
- [ ] API key saved to localStorage
- [ ] API key loaded on page refresh
- [ ] Model selection persists
- [ ] Max iterations slider works

### AI Chat
- [ ] Chat panel visible
- [ ] Disabled when no API key
- [ ] Enabled when API key set
- [ ] Sends request to OpenRouter
- [ ] Shows progress during processing
- [ ] Updates editor on success
- [ ] Shows error on failure
- [ ] Retries up to max iterations

### Offline Mode
- [ ] Detects offline status
- [ ] Shows offline indicator
- [ ] Disables AI features
- [ ] Keeps local features working
- [ ] Re-enables when back online
