use gtk::{
    builders::HeaderBarBuilder,
    prelude::NotebookExtManual,
    traits::{
        BoxExt, CellRendererExt, CellRendererProgressExt, ContainerExt, GtkWindowExt, HeaderBarExt,
        ScrolledWindowExt, TreeViewColumnExt, TreeViewExt, WidgetExt,
    },
    Inhibit, Orientation,
};

#[derive(Debug)]
#[repr(i32)]
// General Information tab configuration
pub enum GeneralColumns {
    Name,
    Hash,
    Structure,
    Size,
    TotalPieces,
    DownloadedPieces,
    Peers,
    ActiveConnections,
    Status,
    Progress,
}

#[derive(Debug)]
#[repr(i32)]
// Download Statistics tab configuration
pub enum StatColumns {
    PeerID,
    PeerIP,
    PeerPort,
    PeerStatus,
    OurStatus,
    DownloadSpeed,
}

pub struct View {
    pub window: gtk::ApplicationWindow, // Main view containing the notebook
    pub notebook: Notebook,             // Notebook containing information tabs
}

impl View {
    pub fn new(application: &gtk::Application) -> Self {
        let window = gtk::ApplicationWindow::new(application);
        window.set_position(gtk::WindowPosition::Center);

        let header_bar = HeaderBarBuilder::new().build();
        header_bar.set_title(Some("BitTorrent Client: Albatros Rustico"));
        window.set_border_width(8);
        header_bar.set_show_close_button(true);
        window.set_titlebar(Some(&header_bar));

        window.show_all();
        window.set_default_size(480, 240);
        window.connect_delete_event(move |window, _| {
            window.close();
            Inhibit(false)
        });

        let notebook = Notebook::new();
        window.add(&notebook.notebook);

        View { window, notebook }
    }
}

pub struct Notebook {
    pub notebook: gtk::Notebook,       // Main notebook containing both tabs
    pub general_info: GeneralInfo,     // General Information tab
    pub download_stats: DownloadStats, // Download Statistics tab
}

impl Notebook {
    pub fn new() -> Self {
        let notebook = gtk::Notebook::new();

        // Creating the General Info tab and appending it to the Notebook
        let general_info = GeneralInfo::new();
        let title_1 = "General Information".to_string();
        let label_1 = gtk::Label::new(Some(&title_1));
        let tab_1 = gtk::Box::new(Orientation::Horizontal, 0);
        tab_1.pack_start(&label_1, false, false, 0);
        tab_1.show_all();
        notebook.append_page(&general_info.container, Some(&tab_1));

        // Creating the Download Statistics tab and appending it to the Notebook
        let download_stats = DownloadStats::new();
        let title_2 = "Download Statistics".to_string();
        let label_2 = gtk::Label::new(Some(&title_2));
        let tab_2 = gtk::Box::new(Orientation::Horizontal, 0);
        tab_2.pack_start(&label_2, false, false, 0);
        tab_2.show_all();
        notebook.append_page(&download_stats.container, Some(&tab_2));

        Notebook {
            notebook,
            general_info,
            download_stats,
        }
    }
}

pub struct GeneralInfo {
    pub container: gtk::ScrolledWindow, // ScrolledWindow containing the General Info TreeView
    pub tree_view: gtk::TreeView,       // TreeView containing the relevant General Info
    pub list_store: gtk::ListStore,     // ListStore used to model the TreeView
}

impl GeneralInfo {
    pub fn new() -> Self {
        // Creating the model and TreeView based on it
        let list_store = create_general_model();
        let tree_view = gtk::TreeView::with_model(&list_store);
        add_general_columns(&tree_view);

        // Creating scrollable box
        let container = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        container.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        container.add(&tree_view);
        container.set_border_width(6);

        GeneralInfo {
            container,
            tree_view,
            list_store,
        }
    }
}

fn create_general_model() -> gtk::ListStore {
    let column_types: [glib::Type; 10] = [
        glib::Type::STRING, // Torrent Name
        glib::Type::STRING, // Torrent Hash
        glib::Type::STRING, // Torrent Structure
        glib::Type::STRING, // Total Size
        glib::Type::U32,    // Total No. of Pieces
        glib::Type::U32,    // Total No. of downloaded Pieces
        glib::Type::U32,    // Total No. of Peers provided by the tracker
        glib::Type::U32,    // Total No. of Peers connected
        glib::Type::STRING, // Download status
        glib::Type::U32,    // Progress bar
    ];

    gtk::ListStore::new(&column_types)
}

fn add_general_columns(tree_view: &gtk::TreeView) {
    // Column for name
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Name");
        column.add_attribute(&renderer, "text", GeneralColumns::Name as i32);
        tree_view.append_column(&column);
    }

    // Column for hash
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Hash");
        column.add_attribute(&renderer, "text", GeneralColumns::Hash as i32);
        tree_view.append_column(&column);
    }

    // Column for structure
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Structure");
        column.add_attribute(&renderer, "text", GeneralColumns::Structure as i32);
        tree_view.append_column(&column);
    }

    // Column for size
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Size (gb)");
        column.add_attribute(&renderer, "text", GeneralColumns::Size as i32);
        tree_view.append_column(&column);
    }

    // Column for total pieces
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("TL Pieces");
        column.add_attribute(&renderer, "text", GeneralColumns::TotalPieces as i32);
        tree_view.append_column(&column);
    }

    // Column for downloaded pieces
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("DL Pieces");
        column.add_attribute(&renderer, "text", GeneralColumns::DownloadedPieces as i32);
        tree_view.append_column(&column);
    }

    // Column for peers
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Peers");
        column.add_attribute(&renderer, "text", GeneralColumns::Peers as i32);
        tree_view.append_column(&column);
    }

    // Column for active connections
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Active Connections");
        column.add_attribute(&renderer, "text", GeneralColumns::ActiveConnections as i32);
        tree_view.append_column(&column);
    }

    // Columns for status
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Status");
        column.add_attribute(&renderer, "text", GeneralColumns::Status as i32);
        tree_view.append_column(&column);
    }

    // Column for progress bar
    {
        let renderer = gtk::CellRendererProgress::new();
        renderer.set_value(0);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Progress");
        column.add_attribute(&renderer, "value", GeneralColumns::Progress as i32);
        tree_view.append_column(&column);
    }
}

pub struct DownloadStats {
    pub container: gtk::ScrolledWindow, // ScrolledWindow containing the Download Stats TreeView
    pub tree_view: gtk::TreeView,       // TreeView containing the relevant Download Stats
    pub list_store: gtk::ListStore,     // ListStore used to model the TreeView
}

impl DownloadStats {
    pub fn new() -> Self {
        // Creating the model and TreeView based on it
        let list_store = create_stats_model();
        let tree_view = gtk::TreeView::with_model(&list_store);
        add_stats_columns(&tree_view);

        // Creating scrollable box
        let container = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        container.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        container.add(&tree_view);
        container.set_border_width(6);

        DownloadStats {
            container,
            tree_view,
            list_store,
        }
    }
}

fn create_stats_model() -> gtk::ListStore {
    let column_types: [glib::Type; 6] = [
        glib::Type::STRING, // Peer ID
        glib::Type::STRING, // Peer IP
        glib::Type::U32,    // Peer Port
        glib::Type::STRING, // Peer's status
        glib::Type::STRING, // Our status
        glib::Type::STRING, // Download speed
    ];

    gtk::ListStore::new(&column_types)
}

fn add_stats_columns(tree_view: &gtk::TreeView) {
    // Column for name
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Peer ID");
        column.add_attribute(&renderer, "text", StatColumns::PeerID as i32);
        tree_view.append_column(&column);
    }

    // Column for hash
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("IP Address");
        column.add_attribute(&renderer, "text", StatColumns::PeerIP as i32);
        tree_view.append_column(&column);
    }

    // Column for port
    {
        let renderer = gtk::CellRendererText::new();
        CellRendererExt::set_alignment(&renderer, 0.5, 0.5);
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Port");
        column.add_attribute(&renderer, "text", StatColumns::PeerPort as i32);
        tree_view.append_column(&column);
    }

    // Column for peer's status
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Peer's status");
        column.add_attribute(&renderer, "text", StatColumns::PeerStatus as i32);
        tree_view.append_column(&column);
    }

    // Column for our status
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Our status");
        column.add_attribute(&renderer, "text", StatColumns::OurStatus as i32);
        tree_view.append_column(&column);
    }

    // Column for download speed
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Download speed");
        column.add_attribute(&renderer, "text", StatColumns::DownloadSpeed as i32);
        tree_view.append_column(&column);
    }
}
