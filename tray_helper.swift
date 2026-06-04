import AppKit

class TrayHelper: NSObject, NSApplicationDelegate {
    let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        setupTray()
        startParentMonitor()
    }
    
    func setupTray() {
        guard let button = statusItem.button else { return }
        
        var imageLoaded = false
        if let imagePath = Bundle.main.path(forResource: "logo", ofType: "png"),
           let image = NSImage(contentsOfFile: imagePath) {
            image.size = NSSize(width: 18, height: 18)
            button.image = image
            imageLoaded = true
        } else if let image = NSImage(contentsOfFile: "logo.png") {
            image.size = NSSize(width: 18, height: 18)
            button.image = image
            imageLoaded = true
        }
        
        if !imageLoaded {
            button.title = "⏰"
        }
        
        let menu = NSMenu()
        
        let showItem = NSMenuItem(title: "Show Focus Flow", action: #selector(showApp), keyEquivalent: "")
        showItem.target = self
        
        let hideItem = NSMenuItem(title: "Hide to Tray", action: #selector(hideApp), keyEquivalent: "")
        hideItem.target = self
        
        let exitItem = NSMenuItem(title: "Exit Application", action: #selector(exitApp), keyEquivalent: "")
        exitItem.target = self
        
        menu.addItem(showItem)
        menu.addItem(hideItem)
        menu.addItem(NSMenuItem.separator())
        menu.addItem(exitItem)
        
        statusItem.menu = menu
    }
    
    func startParentMonitor() {
        let monitorThread = Thread {
            // readLine will block until input is received or EOF is reached (when parent exits)
            while let _ = readLine() {}
            // EOF reached, parent process has exited
            DispatchQueue.main.async {
                NSApplication.shared.terminate(nil)
            }
        }
        monitorThread.start()
    }
    
    @objc func showApp() {
        print("SHOW")
        fflush(stdout)
    }
    
    @objc func hideApp() {
        print("HIDE")
        fflush(stdout)
    }
    
    @objc func exitApp() {
        print("EXIT")
        fflush(stdout)
        NSApplication.shared.terminate(nil)
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let delegate = TrayHelper()
app.delegate = delegate
app.run()
