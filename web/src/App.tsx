import React, { Component } from 'react';
// import logo from './logo.svg';
// import './App.css';

import { AppBar, Box, Button, createStyles, CssBaseline, IconButton, /*FormControl, InputLabel, Select,*/ Menu, MenuItem, /* Grid, */ Theme, Toolbar, Tooltip, Typography, withStyles, WithStyles } from '@material-ui/core';
import MonacoEditor from 'react-monaco-editor';
// import SettingsIcon from '@material-ui/icons/Settings';
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import GitHubIcon from '@material-ui/icons/GitHub';
import SwapVertIcon from '@material-ui/icons/SwapVert';
// import ResizeObserver from 'react-resize-detector';

import PACKAGE from '../package.json';


const TARGET_OPTIONS = ['Dataclass', 'DataclassWithJSON', 'PydanticBaseModel', 'PydanticDataclass', 'TypedDictClass', 'TypedDictInline', 'NestedTypedDict'] as const
type TargetType = (typeof TARGET_OPTIONS)[number]

const styles = (theme: Theme) => createStyles({
  root: {
    display: 'flex',
    flexDirection: 'column',
    width: "100vw", height: "100vh", overflow: 'hidden'
  },
  bifoldContainer: {
    [theme.breakpoints.down('sm')]: { flexDirection: 'column' },
    [theme.breakpoints.up('sm')]: { flexDirection: 'row' },
  },
  editorPane: {
    display: 'flex',
    flexBasis: "50%",
    minWidth: "0px", // override content-width
    minHeight: "0px", // override content-width
  },
  optionPane: {
    margin: theme.spacing(2),
  },
  formControl: {
    margin: theme.spacing(1),
    minWidth: 120,
  },
  selectEmpty: {
    marginTop: theme.spacing(2),
  },
  targetLanguage: { margin: theme.spacing(0, 0.5, 0, 1) },
  subtitleWrapper: {
    margin: theme.spacing(0, 2),
    [theme.breakpoints.down('sm')]: { display: 'none' },
    [theme.breakpoints.up('sm')]: { display: 'inline-block' },
  }
})

interface Props extends WithStyles<typeof styles> { }

interface State {
  targetMenu: any
  targetSelected: TargetType
  output: string
}

class App extends Component<Props, State> {
  inputEditor?: any
  outputEditor?: any
  input = `{"message": "Try to paste some structural JSON here"}`

  constructor(props: Props) {
    super(props)

    this.input = localStorage.getItem(`${PACKAGE.name}-code`) ?? this.input

    let targetCandidates = [window.location.hash.slice(1),
      localStorage.getItem(`${PACKAGE.name}-target`)] as TargetType[]
    let target = targetCandidates.find((t) => TARGET_OPTIONS.includes(t))
    if (!target) {
      target = TARGET_OPTIONS[0]
    }
    updateTargetInHash(target)

    this.state = {
      targetMenu: null,
      targetSelected: target,
      output: "# No input"
    }

    this.inputEditorDidMount = this.inputEditorDidMount.bind(this)
    this.outputEditorDidMount = this.outputEditorDidMount.bind(this)
    this.updateEditorsLayout = this.updateEditorsLayout.bind(this)
    this.handleInput = this.handleInput.bind(this)
    this.handleTargetIconClick = this.handleTargetIconClick.bind(this)
    this.handleTargetMenuClose = this.handleTargetMenuClose.bind(this)
  }

  componentDidMount() {
    window.addEventListener('resize', this.updateEditorsLayout)
    this.doGenerate()
  }

  inputEditorDidMount(editor: any, _monaco: any) {
    editor.focus()
    this.inputEditor = editor
  }

  outputEditorDidMount(editor: any, _monaco: any) {
    this.outputEditor = editor
  }

  updateEditorsLayout() {
    this.inputEditor && this.inputEditor.layout()
    this.outputEditor && this.outputEditor.layout()
  }

  async handleInput(newValue: string, _event: any) {
    localStorage.setItem(`${PACKAGE.name}-code`, newValue)
    this.input = newValue
    this.doGenerate()
  }

  async doGenerate() {
    const { json2type, Target } = await import('../../pkg/json2pyi')
    try {
      // Pre-validate JSON
      // TODO: proper error handling within Rust module
      JSON.parse(this.input)
    }
    catch (_e) {
      return
    }
    try {
      const output = json2type(this.input, Target[this.state.targetSelected])
      output && this.setState({ output })
    } catch (e) {
      this.setState({ output: (e as any).toString() })
      console.error(e)
    }
  }

  handleTargetIconClick(event: any) {
    this.setState({ targetMenu: event.target })
  }

  handleTargetMenuClose(event: any) {
    // console.log(event.currentTarget, event.currentTarget.nodeName);
    // if (event.currentTarget.nodeName === 'A') {
    // console.log(event.currentTarget.target)
    // }
    const newTarget = event.currentTarget.dataset.target ?? TARGET_OPTIONS[0]
    updateTargetInHash(newTarget)
    localStorage.setItem(`${PACKAGE.name}-target`, newTarget)
    this.setState({ targetMenu: null, targetSelected: newTarget })
    this.doGenerate()
  }

  render() {
    const classes = this.props.classes;
    const targetSelected = this.state.targetSelected //localStorage.getItem('targetSelected') ?? TARGET_OPTIONS[0]

    return (
      // <ResizeObserver handleWidth handleHeight onResize={this.updateEditorsLayout}>
      <Box className={classes.root}>
        <CssBaseline />
        <AppBar position='static' color="transparent">
          <Toolbar variant='dense'>
            <Typography variant="h6" sx={{ fontFamily: 'monospace' }}>
              JSON2PYI
            </Typography>
            <Box className={classes.subtitleWrapper}>
              <Typography variant="subtitle2">
                JSON to Python Types
              </Typography>
            </Box>
            {/* <LinearProgress color="secondary" /> */}
            <Box sx={{ flexGrow: 1 }} />
            <Tooltip title="Select Target Language" enterDelay={300}>
              <Button
                color="inherit"
                aria-owns={this.state.targetMenu ? 'target-menu' : undefined}
                aria-haspopup="true"
                onClick={this.handleTargetIconClick}
              // data-ga-event-category="header"
              // data-ga-event-action="language"
              >
                <SwapVertIcon />
                <span className={classes.targetLanguage}>
                  {/* {LANGUAGES_LABEL.filter((language) => language.code === userLanguage)[0].text} */}
                  Python - {targetSelected}
                </span>
                <ExpandMoreIcon fontSize="small" />
              </Button>
            </Tooltip>
            <Menu
              id="target-menu"
              anchorEl={this.state.targetMenu}
              open={Boolean(this.state.targetMenu)}
              onClose={this.handleTargetMenuClose}
            >
              {TARGET_OPTIONS.map((target) => (
                <MenuItem
                  // component="a"
                  data-no-link="true"
                  // href={language.code === 'en' ? canonical : `/${language.code}${canonical}`}
                  key={target}
                  selected={target === targetSelected}
                  onClick={this.handleTargetMenuClose}
                  data-target={target}
                // hrefLang={language.code}
                >
                  Python - {target}
                </MenuItem>
              ))}
            </Menu>
            <Tooltip title={"Project Repo"} enterDelay={300}>
              <IconButton
                component="a"
                color="inherit"
                href={PACKAGE.repository.url}
                data-ga-event-category="header"
                data-ga-event-action="github"
              >
                <GitHubIcon />
              </IconButton>
            </Tooltip>
          </Toolbar>
        </AppBar>
        <Box
          // container
          className={classes.bifoldContainer}
          sx={{ flexGrow: 1, display: 'flex', flexWrap: 'nowrap', minHeight: "0px" }}
        >
          <Box className={classes.editorPane} sx={{ display: 'flex', flexDirection: 'column' }}>
            {/* <Box className={classes.optionPane}>
              <FormControl variant="outlined" className={classes.formControl} sx={{ minWidth: "10em" }}>
                <InputLabel id="demo-simple-select-outlined-label">Target Language</InputLabel>
                <Select
                  labelId="demo-simple-select-outlined-label"
                  id="demo-simple-select-outlined"
                  value={10}
                  // onChange={handleChange}
                  label="Target Language"
                  className={classes.selectEmpty}
                >
                  <MenuItem value="">
                    <em>Python</em>
                  </MenuItem>
                  <MenuItem value={10}>Rust</MenuItem>
                  <MenuItem value={20}>Twenty</MenuItem>
                  <MenuItem value={30}>Thirty</MenuItem>
                </Select>
              </FormControl>
            </Box> */}
            <Box flexGrow={1}>
              <MonacoEditor
                width="100%"
                height="100%"
                language="json"
                theme="vs-light"
                value={this.input}
                // options={{lineNumbersMinChars:3}}
                onChange={this.handleInput}
                editorDidMount={this.inputEditorDidMount}
              />
            </Box>
          </Box>
          <Box className={classes.editorPane}>
            <MonacoEditor
              width="100%"
              height="100%"
              language="python"
              theme="vs-light"
              value={this.state.output}
              // options={{lineNumbersMinChars:3}}
              // onChange={::this.onChange}
              editorDidMount={this.outputEditorDidMount}
            />
          </Box>
        </Box>
      </Box>)
    // </ResizeObserver>

  }
}

// function App() {
//   const classes = useStyles();

//   // const state, setState

//   return <Box className={classes.root}>
//     <CssBaseline />
//     <AppBar position='static'>
//       <Toolbar variant='dense'>
//         <Typography variant="h6">
//           JSON to Type
//         </Typography>
//       </Toolbar>
//     </AppBar>
//     <Grid
//       container
//       sx={{ flexGrow: 1 }}
//       direction="row"
//     >
//       <Grid item xs={12} sm={4} md={6}>
//         <MonacoEditor
//           width="100%"
//           height="100%"
//           language="javascript"
//           theme="vs-light"
//           value={"Source a\n\nba\n\nba\n\nba\n\nba\n\nba\n\nba\n\nba\n\nb"}
//         // options={ }
//         // onChange={::this.onChange}
//         //   editorDidMount={::this.editorDidMount}
//         />
//       </Grid>
//       <Grid item xs={12} sm={8} md={6}>
//         <MonacoEditor
//           width="100%"
//           height="100%"
//           language="javascript"
//           theme="vs-light"
//           value={"Target a\n\nba\n\nba\n\nba\n\nba\n\nba\n\nba\n\nba\n\nb"}
//         // options={ }
//         // onChange={::this.onChange}
//         //   editorDidMount={::this.editorDidMount}
//         />
//       </Grid>
//     </Grid>
//   </Box>
// }

// function App() {
//   return (
//     <div className="App">
//       <header className="App-header">
//         <img src={logo} className="App-logo" alt="logo" />
//         <p>
//           Edit <code>src/App.tsx</code> and save to reload.
//         </p>
//         <a
//           className="App-link"
//           href="https://reactjs.org"
//           target="_blank"
//           rel="noopener noreferrer"
//         >
//           Learn React
//         </a>
//       </header>
//       <MonacoEditor
//         width="800"
//         height="600"
//         language="javascript"
//         theme="vs-light"
//       // valreact  ue={}
//       // options={}
//       // onChange={::this.onChange}
//       // editorDidMount={::this.editorDidMount}
//       />
//     </div>
//   );
// }

const updateTargetInHash = (target: string) => {
  window.history.replaceState({}, '', `#${target}`)
}

export default withStyles(styles)(App);
