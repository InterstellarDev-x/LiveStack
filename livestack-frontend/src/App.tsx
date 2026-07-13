

import './App.css'
import { Button } from './components/ui/button'
import { SidebarProvider , SidebarTrigger } from './components/ui/sidebar'

function App() {
 

  return (
    <>

      <Layout><h1>hello how are you</h1> 
      <Button>CLick me</Button></Layout>
    </>
  )
}

export default App


function Layout({children} : {children : React.ReactNode}){
  return (
      <SidebarProvider>

      <main>
        <SidebarTrigger />
        {children}
      </main>
    </SidebarProvider>
  )
}